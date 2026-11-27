//! Prüfung der erfassten Schritte gegen die Modellierungsregeln von BPMN 2.0.
//!
//! Der Ablauf wird als Graph aus Schritten und Zusammenführungen nachgebildet; daraus ergeben sich
//! Erreichbarkeit und Deadlocks. Dazu kommen die üblichen Konventionen für Bezeichnungen.

use std::collections::{HashMap, HashSet, VecDeque};

use super::model::{ProcessStep, branch_label, child_steps, has_branches};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    /// Das Modell ist nach BPMN 2.0 nicht ausführbar oder widersprüchlich.
    Fehler,
    /// Erlaubt, aber vermutlich nicht gewollt.
    Warnung,
    /// Konvention für gut lesbare Modelle.
    Hinweis,
}

impl Level {
    pub fn label(self) -> &'static str {
        match self {
            Self::Fehler => "Fehler",
            Self::Warnung => "Warnung",
            Self::Hinweis => "Hinweis",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Finding {
    pub level: Level,
    pub message: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Node {
    Step(i64),
    /// Zusammenführung nach der Verzweigung des Schritts.
    Join(i64),
}

const BRANCHES: [&str; 2] = ["Ja", "Nein"];

/// Verknüpft eine Folge von Schritten; `after` ist das Element danach. Liefert den Eingang der Folge.
fn link_sequence(
    all: &[ProcessStep],
    group: &[ProcessStep],
    after: Option<Node>,
    graph: &mut HashMap<Node, Vec<Node>>,
) -> Option<Node> {
    let mut following = after;
    for step in group.iter().rev() {
        let node = Node::Step(step.id);
        let mut successors = Vec::new();
        if has_branches(all, step) {
            let join = Node::Join(step.id);
            // Ein Endpunkt mit Verzweigung endet nach der Zusammenführung.
            let join_successors = if step.is_end { Vec::new() } else { following.into_iter().collect() };
            graph.insert(join, join_successors);
            for branch in BRANCHES {
                let children = child_steps(all, Some(step.id), Some(branch));
                successors.push(link_sequence(all, &children, Some(join), graph).unwrap_or(join));
            }
        } else if !step.is_end {
            successors.extend(following);
        }
        graph.insert(node, successors);
        following = Some(node);
    }
    following
}

/// Endet die Folge mit einem Endereignis, bevor sie die Zusammenführung erreicht?
fn sequence_terminates(all: &[ProcessStep], group: &[ProcessStep]) -> bool {
    group.iter().any(|step| {
        step.is_end
            || has_branches(all, step)
                && BRANCHES.iter().all(|branch| {
                    let children = child_steps(all, Some(step.id), Some(branch));
                    !children.is_empty() && sequence_terminates(all, &children)
                })
    })
}

fn has_terminate_end(group: &[ProcessStep]) -> bool {
    group.iter().any(|s| s.is_end && s.end_event == "Terminierung")
}

/// Alle Schritte in der Reihenfolge der Tabellen (Hauptablauf, darin jeweils die Zweige).
fn document_order<'a>(all: &'a [ProcessStep], parent: Option<i64>, branch: Option<&str>, out: &mut Vec<&'a ProcessStep>) {
    for step in child_steps(all, parent, branch) {
        let step = all.iter().find(|s| s.id == step.id).expect("Schritt aus der Liste");
        out.push(step);
        for branch in BRANCHES {
            document_order(all, Some(step.id), Some(branch), out);
        }
    }
}

/// Prüft alle Schritte eines Prozesses; die Befunde sind nach Schwere und Reihenfolge sortiert.
pub fn check(all: &[ProcessStep]) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut add = |level: Level, message: String| findings.push(Finding { level, message });

    let main = child_steps(all, None, None);
    if main.is_empty() {
        return Vec::new();
    }
    let mut ordered = Vec::new();
    document_order(all, None, None, &mut ordered);

    // Erreichbarkeit vom Startereignis aus.
    let mut graph = HashMap::new();
    link_sequence(all, &main, None, &mut graph);
    let start = all.iter().find(|s| s.is_start).unwrap_or(&main[0]);
    let mut reached = HashSet::from([Node::Step(start.id)]);
    let mut queue = VecDeque::from([Node::Step(start.id)]);
    while let Some(node) = queue.pop_front() {
        for &next in graph.get(&node).into_iter().flatten() {
            if reached.insert(next) {
                queue.push_back(next);
            }
        }
    }

    if !all.iter().any(|s| s.is_start) {
        add(Level::Hinweis, format!("Kein Startpunkt festgelegt – „{}“ gilt als Start.", main[0].activity));
    } else if start.parent_step_id.is_some() {
        add(
            Level::Warnung,
            format!("Der Startpunkt „{}“ liegt in einem Zweig. Ein Prozess beginnt im Hauptablauf.", start.activity),
        );
    }
    if start.is_start && start.start_label.is_empty() {
        add(
            Level::Hinweis,
            "Das Startereignis hat keine Bezeichnung. Ereignisse als Zustand benennen, z. B. „Bestellung eingegangen“."
                .to_string(),
        );
    }

    for step in &ordered {
        let name = &step.activity;
        if !reached.contains(&Node::Step(step.id)) {
            add(
                Level::Fehler,
                format!("„{name}“ ist vom Startereignis aus nicht erreichbar (liegt ein Endpunkt davor?)."),
            );
        }

        // Benennung und Angaben je Element.
        match step.element.as_str() {
            "Zwischenereignis" => {}
            element => {
                if step.role.is_empty() {
                    add(Level::Warnung, format!("„{name}“ ist keiner Rolle (Bahn) zugeordnet."));
                }
                if name.split_whitespace().count() < 2 {
                    add(
                        Level::Hinweis,
                        format!("„{name}“: {element} nach dem Muster Objekt + Verb benennen, z. B. „Rechnung prüfen“."),
                    );
                }
            }
        }
        if step.element == "Teilprozess" && step.called_process_id.is_none() {
            add(Level::Hinweis, format!("Teilprozess „{name}“ verweist auf keinen erfassten Prozess."));
        }
        if step.task_type == "Senden" && step.output.is_empty() {
            add(Level::Hinweis, format!("Sende-Aufgabe „{name}“: die gesendete Nachricht als Output erfassen."));
        }
        if step.task_type == "Empfangen" && step.input.is_empty() {
            add(Level::Hinweis, format!("Empfangs-Aufgabe „{name}“: die erwartete Nachricht als Input erfassen."));
        }
        if step.is_end && step.end_label.is_empty() {
            add(
                Level::Hinweis,
                format!("Das Endereignis nach „{name}“ hat keine Bezeichnung (Zustand, z. B. „Rechnung bezahlt“)."),
            );
        }

        // Verzweigungen.
        if !has_branches(all, step) {
            continue;
        }
        let branches: Vec<Vec<ProcessStep>> =
            BRANCHES.iter().map(|b| child_steps(all, Some(step.id), Some(b))).collect();
        match step.gateway.as_str() {
            "Parallel" => {
                for (branch, children) in BRANCHES.iter().zip(&branches) {
                    if children.is_empty() {
                        add(
                            Level::Warnung,
                            format!(
                                "Parallele Verzweigung nach „{name}“: {} ist leer – ein leerer paralleler Zweig ist überflüssig.",
                                branch_label(true, branch)
                            ),
                        );
                    }
                }
                let ends: Vec<bool> = branches.iter().map(|c| sequence_terminates(all, c)).collect();
                for (i, children) in branches.iter().enumerate() {
                    if ends[i] && !ends[1 - i] && !has_terminate_end(children) {
                        add(
                            Level::Fehler,
                            format!(
                                "Parallele Verzweigung nach „{name}“: {} endet mit einem Endereignis, {} läuft weiter. \
                                 Die parallele Zusammenführung wartet dann vergeblich (Deadlock). \
                                 Ein Terminierungs-Endereignis verwenden oder beide Zweige zusammenführen.",
                                branch_label(true, BRANCHES[i]),
                                branch_label(true, BRANCHES[1 - i]),
                            ),
                        );
                    }
                }
            }
            _ => {
                if step.condition.is_empty() {
                    add(Level::Fehler, format!("Die exklusive Verzweigung nach „{name}“ braucht eine Frage (Bedingung)."));
                } else if !step.condition.ends_with('?') {
                    add(
                        Level::Hinweis,
                        format!("Die Verzweigung „{}“ als Frage formulieren (mit „?“ am Ende).", step.condition),
                    );
                }
                if branches.iter().all(Vec::is_empty) {
                    add(Level::Warnung, format!("Die Verzweigung nach „{name}“ hat in keinem Zweig Schritte."));
                }
            }
        }
    }

    findings.sort_by_key(|f| f.level);
    findings
}
