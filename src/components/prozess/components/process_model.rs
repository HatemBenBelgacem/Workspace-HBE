//! Erzeugt aus den Schritten automatisch ein Diagramm nach BPMN 2.0.
//!
//! Ein Pool pro Prozess, eine Bahn pro Rolle. Schritte werden je nach Element als Aufgabe (mit
//! Aufgabentyp und Wiederholung), Teilprozess oder Zwischenereignis gezeichnet, Verzweigungen als
//! exklusives oder paralleles Gateway mit passender Zusammenführung.

use leptos::prelude::*;

use crate::backend::prozess::model::{ProcessStep, child_steps, has_branches};

/// Breite der Pool-Beschriftung ganz links.
const POOL_W: f64 = 30.0;
const LANE_LABEL_W: f64 = 36.0;
/// Mindesthöhe einer Bahn (ein einzelner Prozessstrom).
const LANE_H: f64 = 150.0;
/// Vertikaler Abstand pro Ebene; die beiden Ströme einer Verzweigung liegen mindestens zwei Ebenen auseinander.
const LEVEL_H: f64 = 80.0;
/// Breite einer Aufgaben-Spalte inkl. Abstand zur nächsten.
const COL_W: f64 = 180.0;
/// Breite einer Spalte mit Zwischenereignis.
const EVENT_COL: f64 = 100.0;
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
/// Zusätzlicher Platz nach einem Endpunkt für das Endereignis.
const END_SPACE: f64 = 60.0;
/// Abstand vom Ereignis-Mittelpunkt zur Kante der Aufgabe.
const EVENT_GAP: f64 = 36.0;
const NO_ROLE: &str = "Ohne Rolle";

enum NodeKind {
    /// Aufgabe, Teilprozess oder Zwischenereignis.
    Step(ProcessStep),
    /// Verzweigung; bei exklusiven mit der Frage.
    Split { question: String, parallel: bool },
    /// Zusammenführung beider Ströme.
    Join { parallel: bool },
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
        match &self.kind {
            NodeKind::Step(step) if step.is_event() => 2.0 * EVENT_R,
            NodeKind::Step(_) => BOX_W,
            NodeKind::Split { .. } => SPLIT_COL,
            NodeKind::Join { .. } => JOIN_COL,
        }
    }

    fn center_x(&self) -> f64 {
        self.x + self.width() / 2.0
    }

    fn half_w(&self) -> f64 {
        match &self.kind {
            NodeKind::Step(step) if step.is_event() => EVENT_R,
            NodeKind::Step(_) => BOX_W / 2.0,
            _ => GW_HALF,
        }
    }

    /// Breite, um die die nächste Spalte nach rechts rückt.
    fn advance(&self) -> f64 {
        match &self.kind {
            NodeKind::Step(step) if step.is_event() => EVENT_COL,
            NodeKind::Step(_) => COL_W,
            _ => self.width(),
        }
    }
}

/// Einer der beiden Ströme einer Verzweigung.
struct Branch {
    split: usize,
    /// Fehlt, wenn beide Zweige mit einem Endpunkt enden.
    join: Option<usize>,
    /// Erster und letzter Schritt; `None`, wenn der Zweig leer ist. Der letzte fehlt, wenn der
    /// Zweig mit einem Endpunkt endet.
    steps: Option<(usize, Option<usize>)>,
    /// Ebene des Stroms (für leere Zweige die Linie in der Bahn der Verzweigung).
    level: i32,
    /// Beschriftung der Kante ("Ja"/"Nein"); bei parallelen Verzweigungen keine.
    label: Option<&'static str>,
}

/// Endereignis nach einem Element.
struct End {
    node: usize,
    event: String,
    label: String,
}

/// Vertikaler Platzbedarf (Ebenen über bzw. unter dem eigenen Strom).
#[derive(Clone, Copy, Default)]
struct Extent {
    above: i32,
    below: i32,
}

/// Abstand der beiden Ströme zur Mitte, sodass sich ihre Teilbäume nicht überlappen.
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

/// Ordnet die Elemente an: Die beiden Ströme einer Verzweigung laufen parallel, der erste oberhalb
/// und der zweite unterhalb, das Gateway steht genau in der Mitte. Bahnen wachsen mit der Zahl der
/// benötigten Ebenen.
struct Layout<'a> {
    all: &'a [ProcessStep],
    nodes: Vec<Node>,
    sequence_edges: Vec<(usize, usize)>,
    branches: Vec<Branch>,
    lanes: Vec<String>,
    /// Kleinste und grösste belegte Ebene je Bahn.
    lane_levels: Vec<(i32, i32)>,
    ends: Vec<End>,
    /// Element des als Startpunkt markierten Schritts.
    start: Option<usize>,
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
            ends: Vec::new(),
            start: None,
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
        self.x += node.advance();
        self.occupy(lane, level);
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    /// Setzt nach dem Element ein Endereignis und schafft Platz dafür.
    fn end_at(&mut self, node: usize, step: &ProcessStep) {
        self.ends.push(End { node, event: step.end_event.clone(), label: step.end_label.clone() });
        self.x += END_SPACE;
    }

    /// Legt eine Folge von Schritten auf einer Ebene an; liefert erstes und letztes Element.
    /// Das letzte fehlt, wenn die Folge mit einem Endpunkt endet.
    fn sequence(&mut self, group: Vec<ProcessStep>, level: i32) -> Option<(usize, Option<usize>)> {
        let mut bounds: Option<(usize, Option<usize>)> = None;
        for step in group {
            let (entry, exit) = self.element(step, level);
            bounds = match bounds {
                Some((first, previous)) => {
                    // Nach einem Endpunkt führt kein Pfeil zum nächsten Schritt.
                    if let Some(previous) = previous {
                        self.sequence_edges.push((previous, entry));
                    }
                    Some((first, exit))
                }
                None => Some((entry, exit)),
            };
        }
        bounds
    }

    /// Liefert Eingang und Ausgang des Elements; der Ausgang fehlt bei einem Endpunkt.
    fn element(&mut self, step: ProcessStep, level: i32) -> (usize, Option<usize>) {
        let lane = self.lane(&step);
        let branching = has_branches(self.all, &step);
        let parallel = step.is_parallel();
        let question = step.condition.clone();
        let id = step.id;
        let info = step.clone();

        let task = self.push(NodeKind::Step(step), lane, level);
        if info.is_start {
            self.start = Some(task);
        }
        if !branching {
            if info.is_end {
                self.end_at(task, &info);
                return (task, None);
            }
            return (task, Some(task));
        }

        let yes_steps = child_steps(self.all, Some(id), Some("Ja"));
        let no_steps = child_steps(self.all, Some(id), Some("Nein"));
        let d = branch_distance(
            sequence_extent(self.all, &yes_steps),
            sequence_extent(self.all, &no_steps),
        );

        let split = self.push(NodeKind::Split { question, parallel }, lane, level);
        self.sequence_edges.push((task, split));

        let branch_x = self.x;
        let yes = self.sequence(yes_steps, level - d);
        let yes_end = self.x;
        self.x = branch_x;
        let no = self.sequence(no_steps, level + d);
        self.x = self.x.max(yes_end);

        // Endet jeder Zweig mit einem Endpunkt, gibt es nichts zusammenzuführen.
        let ends_in = |steps: Option<(usize, Option<usize>)>| matches!(steps, Some((_, None)));
        let join = if ends_in(yes) && ends_in(no) {
            None
        } else {
            Some(self.push(NodeKind::Join { parallel }, lane, level))
        };
        let labels = if parallel { [None, None] } else { [Some("Ja"), Some("Nein")] };
        for ((steps, branch_level), label) in [(yes, level - d), (no, level + d)].into_iter().zip(labels) {
            if steps.is_none() {
                self.occupy(lane, branch_level);
            }
            self.branches.push(Branch { split, join, steps, level: branch_level, label });
        }
        match join {
            Some(join) if info.is_end => {
                self.end_at(join, &info);
                (task, None)
            }
            Some(join) => (task, Some(join)),
            None => (task, None),
        }
    }
}

/// Zeichnet die Schritte als BPMN-2.0-Diagramm.
#[component]
pub fn ProcessModel(name: String, steps: Vec<ProcessStep>) -> impl IntoView {
    let mut layout = Layout::new(&steps);
    let Some((first, last)) = layout.sequence(child_steps(&steps, None, None), 0) else {
        return view! {
            <p class="text-base-content/60">
                "Sobald du Schritte erfasst, erscheint hier das Prozessmodell."
            </p>
        }
        .into_any();
    };
    // Ohne festgelegten Endpunkt endet der Hauptablauf nach seinem letzten Element.
    if let Some(last) = last {
        layout.ends.push(End { node: last, event: String::new(), label: String::new() });
    }
    // Ohne festgelegten Startpunkt beginnt der Prozess beim ersten Schritt.
    let start = layout.start.unwrap_or(first);
    let Layout { nodes, sequence_edges, branches, lanes, lane_levels, ends, x: end_x, .. } = layout;

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

    let height: f64 = lane_heights.iter().sum();

    // Endereignisse rechts der Endpunkte.
    let end_events: Vec<(f64, f64, f64, String, String)> = ends
        .into_iter()
        .map(|end| {
            let (x, y) = right(&nodes[end.node]);
            (x, x + EVENT_GAP, y, end.event, end.label)
        })
        .collect();
    let lanes_width = end_events
        .iter()
        .map(|(_, cx, ..)| cx + EVENT_R)
        .fold(end_x, f64::max)
        + 30.0;
    let width = POOL_W + lanes_width;

    // Startereignis: links des Startpunkts; hat dieser schon einen eingehenden Pfeil, darüber.
    let has_incoming = sequence_edges.iter().any(|&(_, to)| to == start)
        || branches.iter().any(|b| matches!(b.steps, Some((first, _)) if first == start));
    let start_node = &nodes[start];
    let (start_event, start_label) = match &start_node.kind {
        NodeKind::Step(step) if step.is_start => (step.start_event.clone(), step.start_label.clone()),
        _ => (String::new(), String::new()),
    };
    let (start_cx, start_cy, start_path, start_label_view) = if has_incoming {
        let (cx, top) = match &start_node.kind {
            NodeKind::Step(step) if step.is_event() => (start_node.center_x(), y(start_node) - EVENT_R),
            // Rechts oben, weil links oben das Input-Datenobjekt steht.
            _ => (start_node.x + BOX_W - 18.0, y(start_node) - BOX_H / 2.0),
        };
        let cy = top - 24.0;
        let label = event_label(&start_label, cx + EVENT_R + 4.0, cy + 4.0, "start");
        (cx, cy, format!("M {cx} {} V {top}", cy + EVENT_R), label)
    } else {
        let (x, y) = left(start_node);
        let cx = x - EVENT_GAP;
        let label = event_label(&start_label, cx, y + EVENT_R + 12.0, "middle");
        (cx, y, format!("M {} {y} H {x}", cx + EVENT_R), label)
    };

    let lane_views = lanes
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let (top, h) = (lane_tops[i], lane_heights[i]);
            let label_cx = LANE_LABEL_W / 2.0;
            let label_cy = top + h / 2.0;
            view! {
                <rect x="0" y=top width=lanes_width height=h class="fill-base-100 stroke-base-300"/>
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
        let split = &nodes[branch.split];
        match (branch.steps, branch.join) {
            (Some((first, last)), join) => {
                let (tx, ty) = left(&nodes[first]);
                let (sx, sy) = vertex_toward(split, ty);
                let out = if sy == ty {
                    format!("M {sx} {sy} H {tx}")
                } else {
                    format!("M {sx} {sy} V {ty} H {tx}")
                };
                paths.push((out, branch.label.map(|label| (split.center_x() + 6.0, ty - 6.0, label))));

                // Ein Zweig mit Endpunkt führt nicht zurück zur Zusammenführung.
                if let (Some(last), Some(join)) = (last, join) {
                    let join = &nodes[join];
                    let (lx, ly) = right(&nodes[last]);
                    let (jx, jy) = vertex_toward(join, ly);
                    let back = if jy == ly {
                        format!("M {lx} {ly} H {}", join.center_x() - GW_HALF)
                    } else {
                        format!("M {lx} {ly} H {jx} V {jy}")
                    };
                    paths.push((back, None));
                }
            }
            (None, Some(join)) => {
                let join = &nodes[join];
                let ly = level_y(split.lane, branch.level);
                let (sx, sy) = vertex_toward(split, ly);
                let (jx, jy) = vertex_toward(join, ly);
                paths.push((
                    format!("M {sx} {sy} V {ly} H {jx} V {jy}"),
                    branch.label.map(|label| (sx + 6.0, ly - 6.0, label)),
                ));
            }
            // Ein leerer Zweig hat immer eine Zusammenführung.
            (None, None) => {}
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
            NodeKind::Step(step) if step.is_event() => {
                intermediate_event_view(step, node.center_x(), y(node)).into_any()
            }
            NodeKind::Step(step) => task_view(step, node.x, y(node)).into_any(),
            NodeKind::Split { question, parallel } => {
                gateway_view(node.center_x(), y(node), question, *parallel).into_any()
            }
            NodeKind::Join { parallel } => gateway_view(node.center_x(), y(node), "", *parallel).into_any(),
        })
        .collect_view();

    let end_views = end_events
        .into_iter()
        .map(|(x, cx, cy, event, label)| {
            let title = if event.is_empty() { "Endereignis".to_string() } else { format!("Endereignis: {event}") };
            view! {
                <path
                    d=format!("M {x} {cy} H {}", cx - EVENT_R)
                    stroke-width="1.5"
                    marker-end="url(#bpmn-arrow)"
                    class="stroke-base-content"
                />
                <g>
                    <title>{title}</title>
                    <circle cx=cx cy=cy r=EVENT_R stroke-width="4" class="fill-base-100 stroke-error"/>
                    {event_icon(&event, cx, cy, true)}
                </g>
                {event_label(&label, cx, cy + EVENT_R + 14.0, "middle")}
            }
        })
        .collect_view();

    let start_title =
        if start_event.is_empty() { "Startereignis".to_string() } else { format!("Startereignis: {start_event}") };
    let pool_cx = POOL_W / 2.0;
    let pool_cy = height / 2.0;

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

                // Pool mit dem Prozessnamen
                <rect x="0" y="0" width=POOL_W height=height class="fill-base-300 stroke-base-content/40"/>
                <text
                    x=pool_cx
                    y=pool_cy
                    transform=format!("rotate(-90 {pool_cx} {pool_cy})")
                    text-anchor="middle"
                    dominant-baseline="middle"
                    font-size="13"
                    font-weight="700"
                    class="fill-base-content"
                >
                    {truncate(&name, 40)}
                </text>

                <g transform=format!("translate({POOL_W} 0)")>
                    {lane_views}

                    // Startereignis
                    <g>
                        <title>{start_title}</title>
                        <circle cx=start_cx cy=start_cy r=EVENT_R stroke-width="2" class="fill-base-100 stroke-success"/>
                        {event_icon(&start_event, start_cx, start_cy, false)}
                    </g>
                    {start_label_view}
                    <path d=start_path stroke-width="1.5" marker-end="url(#bpmn-arrow)" class="stroke-base-content"/>

                    {edge_views}
                    {node_views}
                    {end_views}
                </g>
                <rect x="0" y="0" width=width height=height fill="none" stroke-width="1.5" class="stroke-base-content/40"/>
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

    // Marker unten in der Mitte: Wiederholung und [+] für Teilprozesse.
    let mut markers: Vec<&str> = Vec::new();
    if !step.loop_type.is_empty() {
        markers.push(step.loop_type.as_str());
    }
    if step.element == "Teilprozess" {
        markers.push("Teilprozess");
    }
    let has_markers = !markers.is_empty();
    let marker_y = top + BOX_H - 11.0;
    let system_y = if has_markers { top + BOX_H - 24.0 } else { top + BOX_H - 10.0 };
    // Text nach oben schieben, wenn unten noch System und Marker stehen.
    let lift = if has_system { 8.0 } else { 0.0 } + if has_markers { 7.0 } else { 0.0 };
    let text_top = cy - (line_count - 1.0) * 7.0 - lift;
    let marker_views = markers
        .iter()
        .enumerate()
        .map(|(i, marker)| {
            let mx = center_x + (i as f64 - (markers.len() as f64 - 1.0) / 2.0) * 16.0;
            marker_view(marker, mx, marker_y)
        })
        .collect_view();

    let kind = match (step.element.as_str(), &step.called_process_name) {
        ("Teilprozess", Some(name)) => format!("Aufruf-Aktivität → {name}"),
        ("Teilprozess", None) => "Teilprozess".to_string(),
        _ if step.task_type.is_empty() => "Aufgabe".to_string(),
        _ => format!("{}-Aufgabe", step.task_type),
    };
    let tooltip = format!(
        "{}\n{kind}\nRolle: {}\nInput: {}\nOutput: {}\nSystem: {}",
        step.activity, step.role, step.input, step.output, step.system
    );
    // Eine Aufruf-Aktivität hat nach BPMN einen dicken Rahmen.
    let border = if step.called_process_id.is_some() { "4" } else { "1.5" };

    view! {
        <g>
            <title>{tooltip}</title>
            <rect x=x y=top width=BOX_W height=BOX_H rx="10" stroke-width=border class="fill-primary/10 stroke-primary"/>
            {task_type_icon(&step.task_type, x + 8.0, top + 6.0)}
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
                <text x=center_x y=system_y text-anchor="middle" font-size="10" font-style="italic" class="fill-base-content/60">
                    {truncate(&step.system, 22)}
                </text>
            })}
            {marker_views}
            <DataObject text=step.input.clone() x=x y=top - 30.0 above=true/>
            <DataObject text=step.output.clone() x=x y=top + BOX_H + 8.0 above=false/>
        </g>
    }
}

/// Zwischenereignis: doppelter Kreis mit Symbol, Bezeichnung darunter.
fn intermediate_event_view(step: &ProcessStep, cx: f64, cy: f64) -> impl IntoView {
    let throwing = matches!(step.event_type.as_str(), "Nachricht senden" | "Signal senden" | "Eskalation");
    let icon = match step.event_type.as_str() {
        "Nachricht empfangen" | "Nachricht senden" => "Nachricht",
        "Signal empfangen" | "Signal senden" => "Signal",
        other => other,
    };
    let lines = wrap(&step.activity, 16, 2);
    let tooltip = format!("{}\nZwischenereignis: {}\nRolle: {}", step.activity, step.event_type, step.role);
    view! {
        <g>
            <title>{tooltip}</title>
            <circle cx=cx cy=cy r=EVENT_R stroke-width="1.5" class="fill-base-100 stroke-warning"/>
            <circle cx=cx cy=cy r=EVENT_R - 3.0 stroke-width="1.5" fill="none" class="stroke-warning"/>
            {event_icon(icon, cx, cy, throwing)}
            <text text-anchor="middle" font-size="10" font-weight="600" class="fill-base-content">
                {lines
                    .into_iter()
                    .enumerate()
                    .map(|(i, line)| view! { <tspan x=cx y=cy + EVENT_R + 13.0 + i as f64 * 12.0>{line}</tspan> })
                    .collect_view()}
            </text>
        </g>
    }
}

/// Beschriftung eines Start- oder Endereignisses.
fn event_label(label: &str, x: f64, y: f64, anchor: &'static str) -> impl IntoView + use<> {
    let lines = wrap(label, 16, 2);
    (!lines.is_empty()).then(|| {
        view! {
            <text text-anchor=anchor font-size="10" class="fill-base-content/80">
                {lines
                    .into_iter()
                    .enumerate()
                    .map(|(i, line)| view! { <tspan x=x y=y + i as f64 * 12.0>{line}</tspan> })
                    .collect_view()}
            </text>
        }
    })
}

/// Symbol im Ereigniskreis; `filled` für auslösende (werfende) Ereignisse und Endereignisse.
fn event_icon(event: &str, cx: f64, cy: f64, filled: bool) -> AnyView {
    let (fill, stroke) = if filled {
        ("fill-base-content", "stroke-base-content")
    } else {
        ("fill-base-100", "stroke-base-content")
    };
    match event {
        "Nachricht" => envelope(cx - 7.0, cy - 5.0, filled).into_any(),
        "Zeitgeber" => view! {
            <circle cx=cx cy=cy r="8" stroke-width="1.2" class="fill-base-100 stroke-base-content"/>
            <path d=format!("M {cx} {} V {cy} H {}", cy - 6.0, cx + 4.0) stroke-width="1.2" fill="none" class="stroke-base-content"/>
        }
        .into_any(),
        "Bedingung" => view! {
            <rect x=cx - 5.0 y=cy - 7.0 width="10" height="14" stroke-width="1.2" class="fill-base-100 stroke-base-content"/>
            <path
                d=format!("M {} {} h 6 M {} {cy} h 6 M {} {} h 6", cx - 3.0, cy - 3.5, cx - 3.0, cx - 3.0, cy + 3.5)
                stroke-width="1"
                class="stroke-base-content"
            />
        }
        .into_any(),
        "Signal" => view! {
            <polygon
                points=format!("{cx},{} {},{} {},{}", cy - 7.0, cx + 7.0, cy + 5.0, cx - 7.0, cy + 5.0)
                stroke-width="1.2"
                class=format!("{fill} {stroke}")
            />
        }
        .into_any(),
        "Eskalation" => view! {
            <path
                d=format!("M {cx} {} L {} {} L {cx} {} L {} {} Z", cy - 8.0, cx + 6.0, cy + 6.0, cy + 1.0, cx - 6.0, cy + 6.0)
                stroke-width="1.2"
                class=format!("{fill} {stroke}")
            />
        }
        .into_any(),
        "Fehler" => view! {
            <path
                d=format!(
                    "M {} {} L {} {} L {} {} L {} {} L {} {} L {} {} Z",
                    cx - 6.0, cy + 7.0, cx - 2.0, cy - 6.0, cx + 2.0, cy + 1.0,
                    cx + 6.0, cy - 7.0, cx + 2.0, cy + 6.0, cx - 2.0, cy - 1.0
                )
                stroke-width="1.2"
                class=format!("{fill} {stroke}")
            />
        }
        .into_any(),
        "Terminierung" => view! { <circle cx=cx cy=cy r="8" class="fill-base-content"/> }.into_any(),
        _ => ().into_any(),
    }
}

/// Briefumschlag; gefüllt für gesendete, leer für empfangene Nachrichten.
fn envelope(x: f64, y: f64, filled: bool) -> impl IntoView {
    let (fill, flap) = if filled {
        ("fill-base-content stroke-base-content", "stroke-base-100")
    } else {
        ("fill-base-100 stroke-base-content", "stroke-base-content")
    };
    view! {
        <rect x=x y=y width="14" height="10" stroke-width="1.2" class=fill/>
        <path d=format!("M {x} {y} L {} {} L {} {y}", x + 7.0, y + 6.0, x + 14.0) fill="none" stroke-width="1.2" class=flap/>
    }
}

/// Symbol des Aufgabentyps oben links in der Aufgabe.
fn task_type_icon(task_type: &str, x: f64, y: f64) -> AnyView {
    match task_type {
        "Benutzer" => view! {
            <circle cx=x + 7.0 cy=y + 4.0 r="3.5" stroke-width="1.2" class="fill-base-100 stroke-base-content"/>
            <path
                d=format!("M {} {} Q {} {} {} {} Z", x + 1.0, y + 15.0, x + 7.0, y + 3.0, x + 13.0, y + 15.0)
                stroke-width="1.2"
                class="fill-base-100 stroke-base-content"
            />
        }
        .into_any(),
        "Manuell" => view! { <text x=x y=y + 12.0 font-size="13" class="fill-base-content">"✋\u{FE0E}"</text> }.into_any(),
        "Service" => view! { <text x=x y=y + 12.0 font-size="14" class="fill-base-content">"⚙\u{FE0E}"</text> }.into_any(),
        "Skript" => view! {
            <path
                d=format!("M {} {y} h 10 l -3 14 h -10 z", x + 3.0)
                stroke-width="1.2"
                class="fill-base-100 stroke-base-content"
            />
            <path
                d=format!("M {} {} h 6 M {} {} h 6 M {} {} h 6", x + 4.0, y + 4.0, x + 3.0, y + 7.0, x + 2.0, y + 10.0)
                stroke-width="1"
                class="stroke-base-content"
            />
        }
        .into_any(),
        "Geschäftsregel" => view! {
            <rect x=x y=y + 1.0 width="15" height="11" stroke-width="1.2" class="fill-base-100 stroke-base-content"/>
            <rect x=x y=y + 1.0 width="15" height="3" class="fill-base-content"/>
            <path
                d=format!("M {x} {} h 15 M {} {} v 8", y + 8.0, x + 5.0, y + 4.0)
                stroke-width="1"
                class="stroke-base-content"
            />
        }
        .into_any(),
        "Senden" => envelope(x, y + 2.0, true).into_any(),
        "Empfangen" => envelope(x, y + 2.0, false).into_any(),
        _ => ().into_any(),
    }
}

/// Marker unten in der Aufgabe (Mittelpunkt `x`, `y`).
fn marker_view(marker: &str, x: f64, y: f64) -> AnyView {
    match marker {
        "Schleife" => view! {
            <path
                d=format!("M {} {} A 5 5 0 1 1 {} {}", x - 3.5, y + 4.0, x + 3.5, y + 4.0)
                fill="none"
                stroke-width="1.3"
                class="stroke-base-content"
            />
            <path d=format!("M {} {} l 0 -4 M {} {} l 4 0", x - 3.5, y + 4.0, x - 3.5, y + 4.0) stroke-width="1.3" class="stroke-base-content"/>
        }
        .into_any(),
        "Mehrfach parallel" => view! {
            <path
                d=format!("M {} {} v 10 M {x} {} v 10 M {} {} v 10", x - 4.0, y - 5.0, y - 5.0, x + 4.0, y - 5.0)
                stroke-width="1.5"
                class="stroke-base-content"
            />
        }
        .into_any(),
        "Mehrfach sequenziell" => view! {
            <path
                d=format!("M {} {} h 10 M {} {y} h 10 M {} {} h 10", x - 5.0, y - 4.0, x - 5.0, x - 5.0, y + 4.0)
                stroke-width="1.5"
                class="stroke-base-content"
            />
        }
        .into_any(),
        "Teilprozess" => view! {
            <rect x=x - 5.0 y=y - 5.0 width="10" height="10" stroke-width="1.2" class="fill-base-100 stroke-base-content"/>
            <path d=format!("M {} {y} h 6 M {x} {} v 6", x - 3.0, y - 3.0) stroke-width="1.2" class="stroke-base-content"/>
        }
        .into_any(),
        _ => ().into_any(),
    }
}

/// Gateway (Raute): exklusiv mit X, parallel mit +. Die Frage steht links oberhalb, damit die
/// senkrechten Linien zu den beiden Strömen frei bleiben.
fn gateway_view(cx: f64, cy: f64, question: &str, parallel: bool) -> impl IntoView {
    let d = 7.0;
    let symbol = if parallel {
        format!("M {} {cy} H {} M {cx} {} V {}", cx - 9.0, cx + 9.0, cy - 9.0, cy + 9.0)
    } else {
        format!("M {} {} L {} {} M {} {} L {} {}", cx - d, cy - d, cx + d, cy + d, cx + d, cy - d, cx - d, cy + d)
    };
    let title = match (parallel, question.is_empty()) {
        (true, _) => "Paralleles Gateway".to_string(),
        (false, true) => "Exklusives Gateway".to_string(),
        (false, false) => format!("Exklusives Gateway: {question}"),
    };
    let lines = wrap(question, 14, 3);
    let line_count = lines.len();
    let label_x = cx - GW_HALF - 6.0;
    let label_bottom = cy - 8.0;
    view! {
        <g>
            <title>{title}</title>
            <polygon
                points=format!("{cx},{} {},{cy} {cx},{} {},{cy}", cy - GW_HALF, cx + GW_HALF, cy + GW_HALF, cx - GW_HALF)
                stroke-width="1.5"
                class="fill-warning/20 stroke-warning"
            />
            <path d=symbol stroke-width=if parallel { "3" } else { "2" } class="stroke-base-content"/>
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

/// Input oder Output als Datenobjekt (Blatt mit Eselsohr) über bzw. unter der Aufgabe, verbunden
/// durch eine gepunktete Datenassoziation.
#[component]
fn DataObject(text: String, x: f64, y: f64, above: bool) -> impl IntoView {
    (!text.is_empty()).then(|| {
        let (w, h, fold) = (14.0, 18.0, 5.0);
        let ox = x + 6.0;
        let page = format!(
            "M {ox} {y} h {} l {fold} {fold} v {} h -{w} z M {} {y} v {fold} h {fold}",
            w - fold,
            h - fold,
            ox + w - fold
        );
        // Assoziation vom Datenobjekt zur Aufgabe (Input) bzw. von der Aufgabe zum Datenobjekt (Output).
        let link = if above {
            format!("M {} {} V {}", ox + w / 2.0, y + h, y + h + 12.0)
        } else {
            format!("M {} {} V {y}", ox + w / 2.0, y - 8.0)
        };
        view! {
            <g>
                <title>{format!("{}: {text}", if above { "Input" } else { "Output" })}</title>
                <path d=link fill="none" stroke-width="1" stroke-dasharray="2 2" class="stroke-base-content/60"/>
                <path d=page stroke-width="1" class="fill-base-100 stroke-base-content/70"/>
                <text x=ox + w + 5.0 y=y + h / 2.0 + 3.0 font-size="10" class="fill-base-content/70">
                    {truncate(&text, 20)}
                </text>
            </g>
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
