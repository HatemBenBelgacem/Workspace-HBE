use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::model::{Entity, EntityColumn};

const BOX_W: f64 = 240.0;
const HEADER_H: f64 = 34.0;
const ROW_H: f64 = 24.0;
const BOX_PAD: f64 = 8.0;
const GAP_X: f64 = 150.0;
const GAP_Y: f64 = 80.0;
const MARGIN: f64 = 40.0;
const MAX_GRID_COLS: usize = 4;
const LOOP_SPACE: f64 = 60.0;

struct Placed {
    entity: Entity,
    columns: Vec<EntityColumn>,
    x: f64,
    y: f64,
}

impl Placed {
    fn height(&self) -> f64 {
        HEADER_H + self.columns.len().max(1) as f64 * ROW_H + BOX_PAD
    }

    fn row_y(&self, row: usize) -> f64 {
        self.y + HEADER_H + row as f64 * ROW_H + ROW_H / 2.0
    }
}

/// Verbindung von einer Spalte (viele Seite) zur referenzierten Entität (eine Seite).
struct Relation {
    path: String,
    /// Punkt am Rand der Entität mit der Spalte und Richtung nach aussen (+1 rechts, -1 links).
    many: (f64, f64, f64),
    /// Punkt am Rand der referenzierten Entität und Richtung nach aussen.
    one: (f64, f64, f64),
    tooltip: String,
}

/// ER-Diagramm in Krähenfuss-Notation: jede Entität als Kasten mit ihren Spalten,
/// Verweise als Linie von der Spalte (N) zur referenzierten Entität (1).
#[component]
pub fn ErDiagram(entities: Vec<Entity>, columns: Vec<EntityColumn>) -> impl IntoView {
    if entities.is_empty() {
        return view! { <p class="text-base-content/60">"Noch keine Entitäten erfasst."</p> }.into_any();
    }

    // Raster: möglichst quadratisch, höchstens vier Spalten breit.
    let grid_cols = ((entities.len() as f64).sqrt().ceil() as usize).clamp(1, MAX_GRID_COLS);
    let mut placed: Vec<Placed> = entities
        .into_iter()
        .map(|entity| {
            let own = columns.iter().filter(|c| c.entity_id == entity.id).cloned().collect();
            Placed { entity, columns: own, x: 0.0, y: 0.0 }
        })
        .collect();

    let mut y = MARGIN;
    for row in placed.chunks_mut(grid_cols) {
        let row_h = row.iter().map(Placed::height).fold(0.0, f64::max);
        for (i, p) in row.iter_mut().enumerate() {
            p.x = MARGIN + i as f64 * (BOX_W + GAP_X);
            p.y = y;
        }
        y += row_h + GAP_Y;
    }
    // Rechts Platz für Verbindungen zwischen übereinanderliegenden Entitäten, die aussen herum laufen.
    let width = MARGIN * 2.0 + grid_cols as f64 * BOX_W + (grid_cols - 1) as f64 * GAP_X + LOOP_SPACE;
    let height = y - GAP_Y + MARGIN;

    let relations: Vec<Relation> = placed
        .iter()
        .flat_map(|source| {
            source.columns.iter().enumerate().filter_map(|(row, column)| {
                let target = placed.iter().find(|p| Some(p.entity.id) == column.references_entity_id)?;
                Some(relation(source, row, column, target))
            })
        })
        .collect();

    let relation_lines = relations
        .iter()
        .map(|r| {
            view! {
                <path d=r.path.clone() fill="none" stroke-width="1.5" class="stroke-base-content/70">
                    <title>{r.tooltip.clone()}</title>
                </path>
            }
        })
        .collect_view();

    let relation_ends = relations
        .iter()
        .map(|r| {
            let (mx, my, md) = r.many;
            let (ox, oy, od) = r.one;
            view! {
                // Krähenfuss: viele
                <path
                    d=format!("M {} {my} L {mx} {} M {} {my} L {mx} {my} M {} {my} L {mx} {}", mx + md * 14.0, my - 7.0, mx + md * 14.0, mx + md * 14.0, my + 7.0)
                    stroke-width="1.5"
                    class="stroke-base-content"
                />
                <text x=mx + md * 22.0 y=my - 6.0 text-anchor="middle" font-size="10" font-weight="600" class="fill-base-content/70">"N"</text>
                // Zwei Striche: genau eine
                <path
                    d=format!("M {} {} V {} M {} {} V {}", ox + od * 8.0, oy - 7.0, oy + 7.0, ox + od * 13.0, oy - 7.0, oy + 7.0)
                    stroke-width="1.5"
                    class="stroke-base-content"
                />
                <text x=ox + od * 22.0 y=oy - 6.0 text-anchor="middle" font-size="10" font-weight="600" class="fill-base-content/70">"1"</text>
            }
        })
        .collect_view();

    let boxes = placed.into_iter().map(entity_box).collect_view();

    view! {
        <div class="overflow-x-auto rounded-box border border-base-300 bg-base-200/40">
            <svg
                xmlns="http://www.w3.org/2000/svg"
                width=width
                height=height
                viewBox=format!("0 0 {width} {height}")
                class="block"
                role="img"
                aria-label="ER-Diagramm"
            >
                {relation_lines}
                {boxes}
                {relation_ends}
            </svg>
        </div>
    }
    .into_any()
}

fn relation(source: &Placed, row: usize, column: &EntityColumn, target: &Placed) -> Relation {
    let sy = source.row_y(row);
    let ty = target.y + HEADER_H / 2.0;
    let (s_left, s_right) = (source.x, source.x + BOX_W);
    let (t_left, t_right) = (target.x, target.x + BOX_W);

    // Seiten so wählen, dass die Linie auf kurzem Weg zwischen den Kästen verläuft;
    // liegen beide übereinander (oder ist es dieselbe Entität), läuft sie rechts aussen herum.
    let ((sx, sd), (tx, td)) = if t_left > s_right {
        ((s_right, 1.0), (t_left, -1.0))
    } else if t_right < s_left {
        ((s_left, -1.0), (t_right, 1.0))
    } else {
        ((s_right, 1.0), (t_right, 1.0))
    };

    let bend = ((tx - sx).abs() / 2.0).max(60.0);
    Relation {
        path: format!(
            "M {sx} {sy} C {} {sy}, {} {ty}, {tx} {ty}",
            sx + sd * bend,
            tx + td * bend
        ),
        many: (sx, sy, sd),
        one: (tx, ty, td),
        tooltip: format!(
            "{}.{} → {} (N:1)",
            source.entity.name, column.name, target.entity.name
        ),
    }
}

fn entity_box(placed: Placed) -> impl IntoView {
    let Placed { entity, columns, x, y } = placed;
    let height = HEADER_H + columns.len().max(1) as f64 * ROW_H + BOX_PAD;
    let href = format!("/entitaeten/{}", entity.id);
    let navigate = use_navigate();

    let rows = if columns.is_empty() {
        view! {
            <text x=x + 12.0 y=y + HEADER_H + ROW_H / 2.0 dominant-baseline="middle" font-size="11" font-style="italic" class="fill-base-content/50">
                "Keine Spalten"
            </text>
        }
        .into_any()
    } else {
        columns
            .into_iter()
            .enumerate()
            .map(|(row, column)| {
                let cy = y + HEADER_H + row as f64 * ROW_H + ROW_H / 2.0;
                let is_reference = column.references_entity_id.is_some();
                let name_x = if is_reference { x + 34.0 } else { x + 12.0 };
                let tooltip = if column.meaning.is_empty() {
                    column.name.clone()
                } else {
                    format!("{}: {}", column.name, column.meaning)
                };
                view! {
                    <g>
                        <title>{tooltip}</title>
                        {(row > 0).then(|| view! {
                            <line x1=x y1=cy - ROW_H / 2.0 x2=x + BOX_W y2=cy - ROW_H / 2.0 class="stroke-base-300"/>
                        })}
                        {is_reference.then(|| view! {
                            <text x=x + 12.0 y=cy dominant-baseline="middle" font-size="9" font-weight="700" class="fill-secondary">"FK"</text>
                        })}
                        <text x=name_x y=cy dominant-baseline="middle" font-size="12" class="fill-base-content">
                            {truncate(&column.name, 20)}
                        </text>
                        <text x=x + BOX_W - 12.0 y=cy text-anchor="end" dominant-baseline="middle" font-size="11" class="fill-base-content/60">
                            {truncate(&column.data_type, 14)}
                        </text>
                    </g>
                }
            })
            .collect_view()
            .into_any()
    };

    view! {
        <g>
            <rect x=x y=y width=BOX_W height=height rx="8" stroke-width="1.5" class="fill-base-100 stroke-primary"/>
            <g class="cursor-pointer" on:click=move |_| navigate(&href, Default::default())>
                <title>"Entität öffnen"</title>
                <path
                    d=format!("M {x} {} Q {x} {y} {} {y} H {} Q {} {y} {} {} V {} H {x} Z", y + 8.0, x + 8.0, x + BOX_W - 8.0, x + BOX_W, x + BOX_W, y + 8.0, y + HEADER_H)
                    class="fill-primary"
                />
                <text x=x + BOX_W / 2.0 y=y + HEADER_H / 2.0 text-anchor="middle" dominant-baseline="middle" font-size="13" font-weight="700" class="fill-primary-content">
                    {truncate(&entity.name, 26)}
                </text>
            </g>
            {rows}
        </g>
    }
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_string()
    } else {
        format!("{}…", text.chars().take(max - 1).collect::<String>())
    }
}
