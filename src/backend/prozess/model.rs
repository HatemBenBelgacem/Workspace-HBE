use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Process {
    pub id: i64,
    pub name: String,
    pub description: String,
}

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
    /// Schritt mit der Bedingung, zu dessen Ja- oder Nein-Zweig dieser Schritt gehört.
    pub parent_step_id: Option<i64>,
    /// "Ja" oder "Nein"; leer bei Schritten des Hauptablaufs.
    pub branch: Option<String>,
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
        }
    }
}

/// Die in der Tabelle bearbeitbaren Felder eines Schritts.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StepFields {
    pub activity: String,
    pub role: String,
    pub input: String,
    pub output: String,
    pub condition: String,
    pub system: String,
}

/// Schritte einer Ebene in ihrer Reihenfolge: Hauptablauf (`parent = None`) oder ein Ja-/Nein-Zweig.
pub fn child_steps(steps: &[ProcessStep], parent: Option<i64>, branch: Option<&str>) -> Vec<ProcessStep> {
    let mut children: Vec<ProcessStep> = steps
        .iter()
        .filter(|s| s.parent_step_id == parent && s.branch.as_deref() == branch)
        .cloned()
        .collect();
    children.sort_by_key(|s| (s.position, s.id));
    children
}
