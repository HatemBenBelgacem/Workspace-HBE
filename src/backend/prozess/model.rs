use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Process {
    pub id: i64,
    pub name: String,
    pub description: String,
}

/// Art des BPMN-Elements, das ein Schritt darstellt.
pub const ELEMENTS: &[&str] = &["Aufgabe", "Teilprozess", "Zwischenereignis"];
/// Aufgabentypen nach BPMN 2.0; leer ist die abstrakte Aufgabe ohne Typ.
pub const TASK_TYPES: &[&str] =
    &["", "Benutzer", "Manuell", "Service", "Skript", "Geschäftsregel", "Senden", "Empfangen"];
/// Zwischenereignisse: eintretende (fangende) und auslösende (werfende).
pub const EVENT_TYPES: &[&str] = &[
    "Zeitgeber",
    "Nachricht empfangen",
    "Nachricht senden",
    "Bedingung",
    "Signal empfangen",
    "Signal senden",
    "Eskalation",
];
/// Wiederholung einer Aufgabe oder eines Teilprozesses; leer ist keine.
pub const LOOP_TYPES: &[&str] = &["", "Schleife", "Mehrfach parallel", "Mehrfach sequenziell"];
/// Verzweigung nach dem Schritt; leer ist keine.
pub const GATEWAYS: &[&str] = &["", "Exklusiv", "Parallel"];
/// Auslöser des Startereignisses; leer ist das Blanko-Startereignis.
pub const START_EVENTS: &[&str] = &["", "Nachricht", "Zeitgeber", "Bedingung", "Signal"];
/// Ergebnis des Endereignisses; leer ist das Blanko-Endereignis.
pub const END_EVENTS: &[&str] = &["", "Nachricht", "Fehler", "Eskalation", "Signal", "Terminierung"];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct ProcessStep {
    pub id: i64,
    pub process_id: i64,
    pub position: i32,
    pub activity: String,
    pub role: String,
    pub input: String,
    pub output: String,
    pub condition: String,
    pub system: String,
    /// Schritt mit der Verzweigung, zu deren Zweig dieser Schritt gehört.
    pub parent_step_id: Option<i64>,
    /// "Ja" oder "Nein" (bei parallelen Verzweigungen Zweig 1 bzw. 2); leer im Hauptablauf.
    pub branch: Option<String>,
    /// Einstiegspunkt des Prozesses (höchstens ein Schritt pro Prozess).
    pub is_start: bool,
    /// Der Prozess endet nach diesem Schritt.
    pub is_end: bool,
    /// Eintrag aus [`ELEMENTS`].
    pub element: String,
    /// Eintrag aus [`TASK_TYPES`], nur bei Aufgaben.
    pub task_type: String,
    /// Eintrag aus [`EVENT_TYPES`], nur bei Zwischenereignissen.
    pub event_type: String,
    /// Eintrag aus [`LOOP_TYPES`].
    pub loop_type: String,
    /// Eintrag aus [`GATEWAYS`].
    pub gateway: String,
    /// Eintrag aus [`START_EVENTS`], nur beim Startpunkt.
    pub start_event: String,
    pub start_label: String,
    /// Eintrag aus [`END_EVENTS`], nur bei Endpunkten.
    pub end_event: String,
    pub end_label: String,
    /// Von einem Teilprozess aufgerufener Prozess.
    pub called_process_id: Option<i64>,
    pub called_process_name: Option<String>,
}

impl ProcessStep {
    pub fn fields(&self) -> StepFields {
        StepFields {
            activity: self.activity.clone(),
            role: self.role.clone(),
            input: self.input.clone(),
            output: self.output.clone(),
            condition: self.condition.clone(),
            system: self.system.clone(),
            is_start: self.is_start,
            is_end: self.is_end,
            element: self.element.clone(),
            task_type: self.task_type.clone(),
            event_type: self.event_type.clone(),
            loop_type: self.loop_type.clone(),
            gateway: self.gateway.clone(),
            start_event: self.start_event.clone(),
            start_label: self.start_label.clone(),
            end_event: self.end_event.clone(),
            end_label: self.end_label.clone(),
            called_process_id: self.called_process_id,
        }
    }

    pub fn is_event(&self) -> bool {
        self.element == "Zwischenereignis"
    }

    pub fn is_parallel(&self) -> bool {
        self.gateway == "Parallel"
    }
}

/// Die im Modalfenster bearbeitbaren Felder eines Schritts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StepFields {
    pub activity: String,
    pub role: String,
    pub input: String,
    pub output: String,
    pub condition: String,
    pub system: String,
    pub is_start: bool,
    pub is_end: bool,
    pub element: String,
    pub task_type: String,
    pub event_type: String,
    pub loop_type: String,
    pub gateway: String,
    pub start_event: String,
    pub start_label: String,
    pub end_event: String,
    pub end_label: String,
    pub called_process_id: Option<i64>,
}

impl Default for StepFields {
    fn default() -> Self {
        Self {
            activity: String::new(),
            role: String::new(),
            input: String::new(),
            output: String::new(),
            condition: String::new(),
            system: String::new(),
            is_start: false,
            is_end: false,
            element: "Aufgabe".to_string(),
            task_type: String::new(),
            event_type: String::new(),
            loop_type: String::new(),
            gateway: String::new(),
            start_event: String::new(),
            start_label: String::new(),
            end_event: String::new(),
            end_label: String::new(),
            called_process_id: None,
        }
    }
}

impl StepFields {
    /// Prüft die Auswahlfelder und entfernt Angaben, die für das gewählte Element nicht gelten.
    pub fn normalized(mut self) -> Result<Self, String> {
        for (value, allowed, label) in [
            (&self.element, ELEMENTS, "Element"),
            (&self.task_type, TASK_TYPES, "Aufgabentyp"),
            (&self.loop_type, LOOP_TYPES, "Wiederholung"),
            (&self.gateway, GATEWAYS, "Verzweigung"),
            (&self.start_event, START_EVENTS, "Startereignis"),
            (&self.end_event, END_EVENTS, "Endereignis"),
        ] {
            if !allowed.contains(&value.as_str()) {
                return Err(format!("Ungültige Angabe für {label}: „{value}“"));
            }
        }
        let is_event = self.element == "Zwischenereignis";
        if is_event && !EVENT_TYPES.contains(&self.event_type.as_str()) {
            return Err("Bitte einen Ereignistyp wählen".to_string());
        }
        if !is_event {
            self.event_type.clear();
        }
        if self.element != "Aufgabe" {
            self.task_type.clear();
        }
        if self.element != "Teilprozess" {
            self.called_process_id = None;
        }
        // Ereignisse lassen sich nicht wiederholen.
        if is_event {
            self.loop_type.clear();
        }
        // Eine Frage gibt es nur bei der exklusiven Verzweigung.
        if self.gateway != "Exklusiv" {
            self.condition.clear();
        }
        if !self.is_start {
            self.start_event.clear();
            self.start_label.clear();
        }
        if !self.is_end {
            self.end_event.clear();
            self.end_label.clear();
        }
        for text in [
            &mut self.activity,
            &mut self.role,
            &mut self.input,
            &mut self.output,
            &mut self.condition,
            &mut self.system,
            &mut self.start_label,
            &mut self.end_label,
        ] {
            *text = text.trim().to_string();
        }
        Ok(self)
    }
}

/// Bezeichnung eines Zweigs für die Anzeige: "Ja"/"Nein" oder bei parallelen Verzweigungen 1/2.
pub fn branch_label(parallel: bool, branch: &str) -> &'static str {
    match (parallel, branch) {
        (true, "Ja") => "Zweig 1",
        (true, _) => "Zweig 2",
        (false, "Ja") => "Ja",
        (false, _) => "Nein",
    }
}

/// Hat der Schritt eine Verzweigung? Auch dann, wenn die Art entfernt wurde, aber noch Zweige bestehen.
pub fn has_branches(all: &[ProcessStep], step: &ProcessStep) -> bool {
    !step.gateway.is_empty() || all.iter().any(|s| s.parent_step_id == Some(step.id))
}

/// Schritte einer Ebene in ihrer Reihenfolge: Hauptablauf (`parent = None`) oder ein Zweig.
pub fn child_steps(steps: &[ProcessStep], parent: Option<i64>, branch: Option<&str>) -> Vec<ProcessStep> {
    let mut children: Vec<ProcessStep> = steps
        .iter()
        .filter(|s| s.parent_step_id == parent && s.branch.as_deref() == branch)
        .cloned()
        .collect();
    children.sort_by_key(|s| (s.position, s.id));
    children
}
