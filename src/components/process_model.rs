use leptos::prelude::*;

use crate::model::{ProcessStep, child_steps};

const LANE_LABEL_W: f64 = 36.0;
/// Mindesthöhe einer Bahn (ein einzelner Prozessstrom).
const LANE_H: f64 = 150.0;
/// Vertikaler Abstand pro Ebene; Ja- und Nein-Strom liegen mindestens zwei Ebenen auseinander.
const LEVEL_H: f64 = 80.0;
/// Breite einer Aufgaben-Spalte inkl. Abstand zur nächsten.
const COL_W: f64 = 180.0;
const BOX_W: f64 = 140.0;
const BOX_H: f64 = 72.0;
const EVENT_R: f64 = 14.0;
const FIRST_X: f64 = LANE_LABEL_W + 80.0;
/// Halbe Diagonale der Gateway-Raute.
const GW_HALF: f64 = 20.0;
/// Spalte der Verzweigung; links der Raute steht die Frage.
const SPLIT_COL: f64 = 140.0;
/// Spalte der Zusammenführung.
const JOIN_COL: f64 = 80.0;
const NO_ROLE: &str = "Ohne Rolle";

enum NodeKind {
    Task(ProcessStep),
    /// Verzweigung mit der Frage aus "Bedingung".
    Split(String),
    /// Zusammenführung von Ja- und Nein-Strom.
    Join,
}

/// Ein Element im Diagramm. Die y-Position ergibt sich erst, wenn die Höhe aller Bahnen bekannt ist:
/// Bahn (Rolle) plus Ebene innerhalb der Bahn.
struct Node {
    kind: NodeKind,
    /// Linke Kante der Spalte.
    x: f64,
    lane: usize,
    level: i32,
}

impl Node {
    fn width(&self) -> f64 {
        match self.kind {
            NodeKind::Task(_) => BOX_W,
            NodeKind::Split(_) => SPLIT_COL,
            NodeKind::Join => JOIN_COL,
        }
    }

    fn center_x(&self) -> f64 {
        self.x + self.width() / 2.0
    }

    fn half_w(&self) -> f64 {
        if matches!(self.kind, NodeKind::Task(_)) { BOX_W / 2.0 } else { GW_HALF }
    }
}

/// Ja- oder Nein-Strom einer Verzweigung.
struct Branch {
    split: usize,
    join: usize,
    /// Erster und letzter Schritt; `None`, wenn der Zweig leer ist.
    steps: Option<(usize, usize)>,
    /// Ebene des Stroms (für leere Zweige die Linie in der Bahn der Verzweigung).
    level: i32,
    label: &'static str,
}

/// Vertikaler Platzbedarf (Ebenen über bzw. unter dem eigenen Strom).
#[derive(Clone, Copy, Default)]
struct Extent {
    above: i32,
    below: i32,
}

fn has_branches(all: &[ProcessStep], step: &ProcessStep) -> bool {
    !step.condition.trim().is_empty() || all.iter().any(|s| s.parent_step_id == Some(step.id))
}

/// Abstand von Ja- bzw. Nein-Strom zur Mitte, sodass sich ihre Teilbäume nicht überlappen.
fn branch_distance(yes: Extent, no: Extent) -> i32 {
    yes.below.max(no.above) + 1
}

fn sequence_extent(all: &[ProcessStep], group: &[ProcessStep]) -> Extent {
    group.iter().fold(Extent::default(), |acc, step| {
        let e = element_extent(all, step);
        Extent { above: acc.above.max(e.above), below: acc.below.max(e.below) }
    })
}

fn element_extent(all: &[ProcessStep], step: &ProcessStep) -> Extent {
    if !has_branches(all, step) {
        return Extent::default();
    }
    let yes = sequence_extent(all, &child_steps(all, Some(step.id), Some("Ja")));
    let no = sequence_extent(all, &child_steps(all, Some(step.id), Some("Nein")));
    let d = branch_distance(yes, no);
    Extent { above: d + yes.above, below: d + no.below }
}

/// Ordnet die Elemente an: Ja- und Nein-Strom laufen parallel, Ja oberhalb und Nein unterhalb,
/// die Verzweigung steht genau in der Mitte. Bahnen wachsen mit der Zahl der benötigten Ebenen.
struct Layout<'a> {
    all: &'a [ProcessStep],
    nodes: Vec<Node>,
    sequence_edges: Vec<(usize, usize)>,
    branches: Vec<Branch>,
    lanes: Vec<String>,
    /// Kleinste und grösste belegte Ebene je Bahn.
    lane_levels: Vec<(i32, i32)>,
    x: f64,
}

impl<'a> Layout<'a> {
    fn new(all: &'a [ProcessStep]) -> Self {
        Self {
            all,
            nodes: Vec::new(),
            sequence_edges: Vec::new(),
            branches: Vec::new(),
            lanes: Vec::new(),
            lane_levels: Vec::new(),
            x: FIRST_X,
        }
    }

    fn lane(&mut self, step: &ProcessStep) -> usize {
        let name = lane_name(step);
        match self.lanes.iter().position(|l| *l == name) {
            Some(index) => index,
            None => {
                self.lanes.push(name);
                self.lane_levels.push((0, 0));
                self.lanes.len() - 1
            }
        }
    }

    fn occupy(&mut self, lane: usize, level: i32) {
        let (min, max) = &mut self.lane_levels[lane];
        *min = (*min).min(level);
        *max = (*max).max(level);
    }

    fn push(&mut self, kind: NodeKind, lane: usize, level: i32) -> usize {
        let node = Node { kind, x: self.x, lane, level };
        self.x += match node.kind {
            NodeKind::Task(_) => COL_W,
            _ => node.width(),
        };
        self.occupy(lane, level);
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    /// Legt eine Folge von Schritten auf einer Ebene an; liefert erstes und letztes Element.
    fn sequence(&mut self, group: Vec<ProcessStep>, level: i32) -> Option<(usize, usize)> {
        let mut bounds: Option<(usize, usize)> = None;
        for step in group {
            let (entry, exit) = self.element(step, level);
            bounds = match bounds {
                Some((first, previous)) => {
                    self.sequence_edges.push((previous, entry));
                    Some((first, exit))
                }
                None => Some((entry, exit)),
            };
        }
        bounds
    }

    fn element(&mut self, step: ProcessStep, level: i32) -> (usize, usize) {
        let lane = self.lane(&step);
        if !has_branches(self.all, &step) {
            let task = self.push(NodeKind::Task(step), lane, level);
            return (task, task);
        }

        let yes_steps = child_steps(self.all, Some(step.id), Some("Ja"));
        let no_steps = child_steps(self.all, Some(step.id), Some("Nein"));
        let d = branch_distance(
            sequence_extent(self.all, &yes_steps),
            sequence_extent(self.all, &no_steps),
        );

        let condition = step.condition.clone();
        let task = self.push(NodeKind::Task(step), lane, level);
        let split = self.push(NodeKind::Split(condition), lane, level);
        self.sequence_edges.push((task, split));

        let branch_x = self.x;
        let yes = self.sequence(yes_steps, level - d);
        let yes_end = self.x;
        self.x = branch_x;
        let no = self.sequence(no_steps, level + d);
        self.x = self.x.max(yes_end);

        let join = self.push(NodeKind::Join, lane, level);
        for (steps, branch_level, label) in [(yes, level - d, "Ja"), (no, level + d, "Nein")] {
            if steps.is_none() {
                self.occupy(lane, branch_level);
            }
            self.branches.push(Branch { split, join, steps, level: branch_level, label });
        }
        (task, join)
    }
}

/// Zeichnet die Schritte als BPMN-ähnliches Modell: eine Schwimmbahn pro Rolle, eine Aufgabe
/// pro Schritt und bei Bedingungen eine Verzweigung in parallelen Ja- und Nein-Strom.
#[component]
pub fn ProcessModel(steps: Vec<ProcessStep>) -> impl IntoView {
    let mut layout = Layout::new(&steps);
    let Some((first, last)) = layout.sequence(child_steps(&steps, None, None), 0) else {
        return view! {
            <p class="text-base-content/60">
                "Sobald du Schritte erfasst, erscheint hier das Prozessmodell."
            </p>
        }
        .into_any();
    };
    let Layout { nodes, sequence_edges, branches, lanes, lane_levels, x: end_x, .. } = layout;

    // Bahnhöhen: eine Grundhöhe plus eine Ebene je zusätzlichem Strom.
    let lane_heights: Vec<f64> = lane_levels
        .iter()
        .map(|&(min, max)| LANE_H + (max - min) as f64 * LEVEL_H)
        .collect();
    let lane_tops: Vec<f64> = lane_heights
        .iter()
        .scan(0.0, |top, h| {
            let current = *top;
            *top += h;
            Some(current)
        })
        .collect();
    let level_y = |lane: usize, level: i32| {
        lane_tops[lane] + LANE_H / 2.0 + (level - lane_levels[lane].0) as f64 * LEVEL_H
    };
    let y = |node: &Node| level_y(node.lane, node.level);
    let left = |node: &Node| (node.center_x() - node.half_w(), y(node));
    let right = |node: &Node| (node.center_x() + node.half_w(), y(node));

    let width = end_x + 10.0 + EVENT_R + 30.0;
    let height: f64 = lane_heights.iter().sum();

    let start_cx = LANE_LABEL_W + 36.0;
    let (first_x, first_y) = left(&nodes[first]);
    let (last_x, last_y) = right(&nodes[last]);
    let end_cx = end_x + 10.0;

    let lane_views = lanes
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let (top, h) = (lane_tops[i], lane_heights[i]);
            let label_cx = LANE_LABEL_W / 2.0;
            let label_cy = top + h / 2.0;
            view! {
                <rect x="0" y=top width=width height=h class="fill-base-100 stroke-base-300"/>
                <rect x="0" y=top width=LANE_LABEL_W height=h class="fill-base-200 stroke-base-300"/>
                <text
                    x=label_cx
                    y=label_cy
                    transform=format!("rotate(-90 {label_cx} {label_cy})")
                    text-anchor="middle"
                    dominant-baseline="middle"
                    font-size="12"
                    font-weight="600"
                    class="fill-base-content"
                >
                    {truncate(name, 18)}
                </text>
            }
        })
        .collect_view();

    // Von einem Gateway aus senkrecht über die obere bzw. untere Spitze zur Ebene `ty`.
    let vertex_toward = |gateway: &Node, ty: f64| {
        let (gx, gy) = (gateway.center_x(), y(gateway));
        if ty < gy {
            (gx, gy - GW_HALF)
        } else if ty > gy {
            (gx, gy + GW_HALF)
        } else {
            (gx + GW_HALF, gy)
        }
    };

    let mut paths: Vec<(String, Option<(f64, f64, &'static str)>)> = sequence_edges
        .iter()
        .map(|&(from, to)| {
            let (sx, sy) = right(&nodes[from]);
            let (tx, ty) = left(&nodes[to]);
            let mid_x = sx + (tx - sx) / 2.0;
            (format!("M {sx} {sy} H {mid_x} V {ty} H {tx}"), None)
        })
        .collect();

    for branch in &branches {
        let (split, join) = (&nodes[branch.split], &nodes[branch.join]);
        let join_x = join.center_x();
        match branch.steps {
            Some((first, last)) => {
                let (tx, ty) = left(&nodes[first]);
                let (sx, sy) = vertex_toward(split, ty);
                let out = if sy == ty {
                    format!("M {sx} {sy} H {tx}")
                } else {
                    format!("M {sx} {sy} V {ty} H {tx}")
                };
                paths.push((out, Some((split.center_x() + 6.0, ty - 6.0, branch.label))));

                let (lx, ly) = right(&nodes[last]);
                let (jx, jy) = vertex_toward(join, ly);
                let back = if jy == ly {
                    format!("M {lx} {ly} H {}", join_x - GW_HALF)
                } else {
                    format!("M {lx} {ly} H {jx} V {jy}")
                };
                paths.push((back, None));
            }
            None => {
                let ly = level_y(split.lane, branch.level);
                let (sx, sy) = vertex_toward(split, ly);
                let (jx, jy) = vertex_toward(join, ly);
                paths.push((
                    format!("M {sx} {sy} V {ly} H {jx} V {jy}"),
                    Some((sx + 6.0, ly - 6.0, branch.label)),
                ));
            }
        }
    }

    let edge_views = paths
        .into_iter()
        .map(|(path, label)| {
            view! {
                <path
                    d=path
                    fill="none"
                    stroke-width="1.5"
                    marker-end="url(#bpmn-arrow)"
                    class="stroke-base-content"
                />
                {label.map(|(lx, ly, text)| view! {
                    <text x=lx y=ly font-size="10" font-weight="600" class="fill-base-content/80">{text}</text>
                })}
            }
        })
        .collect_view();

    let node_views = nodes
        .iter()
        .map(|node| match &node.kind {
            NodeKind::Task(step) => task_view(step, node.x, y(node)).into_any(),
            NodeKind::Split(condition) => gateway_view(node.center_x(), y(node), condition).into_any(),
            NodeKind::Join => gateway_view(node.center_x(), y(node), "").into_any(),
        })
        .collect_view();

    view! {
        <div class="overflow-x-auto rounded-box border border-base-300">
            <svg
                xmlns="http://www.w3.org/2000/svg"
                width=width
                height=height
                viewBox=format!("0 0 {width} {height}")
                class="block"
                role="img"
                aria-label="Prozessmodell"
            >
                <defs>
                    <marker id="bpmn-arrow" viewBox="0 0 10 10" refX="10" refY="5" markerWidth="8" markerHeight="8" orient="auto-start-reverse">
                        <path d="M 0 0 L 10 5 L 0 10 z" class="fill-base-content"/>
                    </marker>
                </defs>

                {lane_views}

                // Startereignis
                <circle cx=start_cx cy=first_y r=EVENT_R stroke-width="2" class="fill-base-100 stroke-success"/>
                <path
                    d=format!("M {} {first_y} H {first_x}", start_cx + EVENT_R)
                    stroke-width="1.5"
                    marker-end="url(#bpmn-arrow)"
                    class="stroke-base-content"
                />

                {edge_views}
                {node_views}

                // Endereignis
                <path
                    d=format!("M {last_x} {last_y} H {}", end_cx - EVENT_R)
                    stroke-width="1.5"
                    marker-end="url(#bpmn-arrow)"
                    class="stroke-base-content"
                />
                <circle cx=end_cx cy=last_y r=EVENT_R stroke-width="4" class="fill-base-100 stroke-error"/>
            </svg>
        </div>
    }
    .into_any()
}

fn task_view(step: &ProcessStep, x: f64, cy: f64) -> impl IntoView {
    let top = cy - BOX_H / 2.0;
    let center_x = x + BOX_W / 2.0;
    let lines = wrap(&step.activity, 17, 3);
    let line_count = lines.len() as f64;
    let has_system = !step.system.is_empty();
    // Text etwas nach oben schieben, wenn unten noch das System steht.
    let text_top = cy - (line_count - 1.0) * 7.0 - if has_system { 8.0 } else { 0.0 };
    let tooltip = format!(
        "{}\nRolle: {}\nInput: {}\nOutput: {}\nSystem: {}",
        step.activity, step.role, step.input, step.output, step.system
    );

    view! {
        <g>
            <title>{tooltip}</title>
            <rect x=x y=top width=BOX_W height=BOX_H rx="10" stroke-width="1.5" class="fill-primary/10 stroke-primary"/>
            <text text-anchor="middle" font-size="12" font-weight="600" class="fill-base-content">
                {lines
                    .into_iter()
                    .enumerate()
                    .map(|(i, line)| {
                        view! {
                            <tspan x=center_x y=text_top + i as f64 * 14.0 dominant-baseline="middle">{line}</tspan>
                        }
                    })
                    .collect_view()}
            </text>
            {has_system.then(|| view! {
                <text x=center_x y=top + BOX_H - 10.0 text-anchor="middle" font-size="10" class="fill-base-content/60">
                    {format!("⚙ {}", truncate(&step.system, 20))}
                </text>
            })}
            <DataLabel text=step.input.clone() prefix="In" x=center_x y=top - 10.0/>
            <DataLabel text=step.output.clone() prefix="Out" x=center_x y=top + BOX_H + 18.0/>
        </g>
    }
}

/// Exklusives Gateway (Raute mit X); `condition` steht als Frage links oberhalb, damit die
/// senkrechten Linien zum Ja- und Nein-Strom frei bleiben.
fn gateway_view(cx: f64, cy: f64, condition: &str) -> impl IntoView {
    let d = 7.0;
    let lines = wrap(condition, 14, 3);
    let line_count = lines.len();
    let label_x = cx - GW_HALF - 6.0;
    let label_bottom = cy - 8.0;
    view! {
        <g>
            <title>{condition.to_string()}</title>
            <polygon
                points=format!("{cx},{} {},{cy} {cx},{} {},{cy}", cy - GW_HALF, cx + GW_HALF, cy + GW_HALF, cx - GW_HALF)
                stroke-width="1.5"
                class="fill-warning/20 stroke-warning"
            />
            <path
                d=format!("M {} {} L {} {} M {} {} L {} {}", cx - d, cy - d, cx + d, cy + d, cx + d, cy - d, cx - d, cy + d)
                stroke-width="2"
                class="stroke-base-content"
            />
            <text text-anchor="end" font-size="10" font-style="italic" class="fill-base-content/80">
                {lines
                    .into_iter()
                    .enumerate()
                    .map(|(n, line)| {
                        view! { <tspan x=label_x y=label_bottom - (line_count - 1 - n) as f64 * 12.0>{line}</tspan> }
                    })
                    .collect_view()}
            </text>
        </g>
    }
}

/// Input oder Output als kleine Beschriftung über bzw. unter der Aufgabe.
#[component]
fn DataLabel(text: String, prefix: &'static str, x: f64, y: f64) -> impl IntoView {
    (!text.is_empty()).then(|| {
        view! {
            <text x=x y=y text-anchor="middle" font-size="10" class="fill-base-content/70">
                <tspan font-weight="600">{format!("{prefix}: ")}</tspan>
                {truncate(&text, 24)}
            </text>
        }
    })
}

fn lane_name(step: &ProcessStep) -> String {
    if step.role.trim().is_empty() {
        NO_ROLE.to_string()
    } else {
        step.role.trim().to_string()
    }
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_string()
    } else {
        format!("{}…", text.chars().take(max - 1).collect::<String>())
    }
}

/// Bricht Text an Wortgrenzen um; was nicht in `max_lines` passt, wird gekürzt.
fn wrap(text: &str, width: usize, max_lines: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let needed = if current.is_empty() { 0 } else { 1 } + word.chars().count();
        if !current.is_empty() && current.chars().count() + needed > width {
            lines.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.len() > max_lines {
        let rest = lines[max_lines - 1..].join(" ");
        lines.truncate(max_lines - 1);
        lines.push(rest);
    }
    lines.into_iter().map(|line| truncate(&line, width)).collect()
}
