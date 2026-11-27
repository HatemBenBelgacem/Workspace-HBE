use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_params_map};

use super::process_model::ProcessModel;
use crate::backend::prozess::bpmn::{self, Level};
use crate::backend::prozess::model::{
    END_EVENTS, EVENT_TYPES, ELEMENTS, GATEWAYS, LOOP_TYPES, Process, ProcessStep, START_EVENTS,
    StepFields, TASK_TYPES, branch_label, child_steps, has_branches,
};
use crate::backend::prozess::server_fns::{
    AddStep, DeleteStep, MoveStep, UpdateStep, get_process, list_processes, list_steps,
};
use crate::components::modal::FormModal;

#[derive(Clone, Copy, PartialEq)]
enum DetailTab {
    Description,
    Model,
}

impl DetailTab {
    fn label(self) -> &'static str {
        match self {
            Self::Description => "Prozessbeschreibung",
            Self::Model => "Prozessmodell",
        }
    }
}

/// Alle Aktionen, die die Tabellen auf dieser Seite auslösen können.
#[derive(Clone, Copy)]
struct StepActions {
    add: ServerAction<AddStep>,
    update: ServerAction<UpdateStep>,
    delete: ServerAction<DeleteStep>,
    move_step: ServerAction<MoveStep>,
}

/// Eingabefelder des Schritt-Modalfensters, für neue und bearbeitete Schritte.
#[derive(Clone, Copy)]
struct StepDraft {
    activity: RwSignal<String>,
    role: RwSignal<String>,
    input: RwSignal<String>,
    output: RwSignal<String>,
    condition: RwSignal<String>,
    system: RwSignal<String>,
    is_start: RwSignal<bool>,
    is_end: RwSignal<bool>,
    element: RwSignal<String>,
    task_type: RwSignal<String>,
    event_type: RwSignal<String>,
    loop_type: RwSignal<String>,
    gateway: RwSignal<String>,
    start_event: RwSignal<String>,
    start_label: RwSignal<String>,
    end_event: RwSignal<String>,
    end_label: RwSignal<String>,
    called_process_id: RwSignal<Option<i64>>,
}

impl StepDraft {
    fn new() -> Self {
        let f = StepFields::default();
        Self {
            activity: RwSignal::new(f.activity),
            role: RwSignal::new(f.role),
            input: RwSignal::new(f.input),
            output: RwSignal::new(f.output),
            condition: RwSignal::new(f.condition),
            system: RwSignal::new(f.system),
            is_start: RwSignal::new(f.is_start),
            is_end: RwSignal::new(f.is_end),
            element: RwSignal::new(f.element),
            task_type: RwSignal::new(f.task_type),
            event_type: RwSignal::new(f.event_type),
            loop_type: RwSignal::new(f.loop_type),
            gateway: RwSignal::new(f.gateway),
            start_event: RwSignal::new(f.start_event),
            start_label: RwSignal::new(f.start_label),
            end_event: RwSignal::new(f.end_event),
            end_label: RwSignal::new(f.end_label),
            called_process_id: RwSignal::new(f.called_process_id),
        }
    }

    fn fields(&self) -> StepFields {
        StepFields {
            activity: self.activity.get_untracked(),
            role: self.role.get_untracked(),
            input: self.input.get_untracked(),
            output: self.output.get_untracked(),
            condition: self.condition.get_untracked(),
            system: self.system.get_untracked(),
            is_start: self.is_start.get_untracked(),
            is_end: self.is_end.get_untracked(),
            element: self.element.get_untracked(),
            task_type: self.task_type.get_untracked(),
            event_type: self.event_type.get_untracked(),
            loop_type: self.loop_type.get_untracked(),
            gateway: self.gateway.get_untracked(),
            start_event: self.start_event.get_untracked(),
            start_label: self.start_label.get_untracked(),
            end_event: self.end_event.get_untracked(),
            end_label: self.end_label.get_untracked(),
            called_process_id: self.called_process_id.get_untracked(),
        }
    }

    fn set(&self, f: StepFields) {
        self.activity.set(f.activity);
        self.role.set(f.role);
        self.input.set(f.input);
        self.output.set(f.output);
        self.condition.set(f.condition);
        self.system.set(f.system);
        self.is_start.set(f.is_start);
        self.is_end.set(f.is_end);
        self.element.set(f.element);
        self.task_type.set(f.task_type);
        self.event_type.set(f.event_type);
        self.loop_type.set(f.loop_type);
        self.gateway.set(f.gateway);
        self.start_event.set(f.start_event);
        self.start_label.set(f.start_label);
        self.end_event.set(f.end_event);
        self.end_label.set(f.end_label);
        self.called_process_id.set(f.called_process_id);
    }

    /// Ein Zwischenereignis braucht einen Ereignistyp.
    fn is_valid(&self) -> bool {
        !self.activity.get().trim().is_empty()
            && (self.element.get() != "Zwischenereignis" || !self.event_type.get().is_empty())
    }
}

/// Wohin ein Schritt aus dem Modalfenster gespeichert wird.
#[derive(Clone, PartialEq)]
enum StepTarget {
    /// Neuer Schritt im Hauptablauf (`parent_step_id` leer) oder in einem Zweig einer Verzweigung.
    New { parent_step_id: Option<i64>, branch: Option<String>, label: Option<&'static str> },
    Edit(i64),
}

/// Das eine Modalfenster der Seite für neue und bearbeitete Schritte.
#[derive(Clone, Copy)]
struct StepForm {
    draft: StepDraft,
    open: RwSignal<bool>,
    target: RwSignal<StepTarget>,
}

impl StepForm {
    fn new() -> Self {
        Self {
            draft: StepDraft::new(),
            open: RwSignal::new(false),
            target: RwSignal::new(StepTarget::New { parent_step_id: None, branch: None, label: None }),
        }
    }

    fn new_step(&self, parent_step_id: Option<i64>, branch: Option<&str>, label: Option<&'static str>) {
        self.draft.set(StepFields::default());
        self.target.set(StepTarget::New { parent_step_id, branch: branch.map(str::to_string), label });
        self.open.set(true);
    }

    fn edit(&self, step: &ProcessStep) {
        self.draft.set(step.fields());
        self.target.set(StepTarget::Edit(step.id));
        self.open.set(true);
    }

    fn title(&self) -> String {
        match self.target.get() {
            StepTarget::Edit(_) => "Schritt bearbeiten".to_string(),
            StepTarget::New { label: Some(label), .. } => format!("Neuer Schritt im Zweig „{label}“"),
            StepTarget::New { .. } => "Neuer Schritt".to_string(),
        }
    }
}

#[component]
pub fn ProcessDetailPage() -> impl IntoView {
    let params = use_params_map();
    let process_id = move || {
        params
            .read()
            .get("id")
            .and_then(|id| id.parse::<i64>().ok())
            .unwrap_or_default()
    };

    let actions = StepActions {
        add: ServerAction::new(),
        update: ServerAction::new(),
        delete: ServerAction::new(),
        move_step: ServerAction::new(),
    };

    let process = Resource::new(process_id, get_process);
    // Andere Prozesse, die ein Teilprozess aufrufen kann.
    let processes = Resource::new(|| (), |_| list_processes());
    let steps = Resource::new(
        move || {
            (
                process_id(),
                actions.add.version().get(),
                actions.update.version().get(),
                actions.delete.version().get(),
                actions.move_step.version().get(),
            )
        },
        |(id, ..)| list_steps(id),
    );

    let active_tab = RwSignal::new(DetailTab::Description);

    let form = StepForm::new();
    let save_step = move || {
        if !form.draft.is_valid() {
            return;
        }
        let fields = form.draft.fields();
        match form.target.get_untracked() {
            StepTarget::Edit(id) => {
                actions.update.dispatch(UpdateStep { id, fields });
            }
            StepTarget::New { parent_step_id, branch, .. } => {
                actions.add.dispatch(AddStep { process_id: process_id(), parent_step_id, branch, fields });
            }
        }
        form.open.set(false);
    };

    let error = move || {
        [
            actions.add.value().get().and_then(Result::err),
            actions.update.value().get().and_then(Result::err),
            actions.delete.value().get().and_then(Result::err),
            actions.move_step.value().get().and_then(Result::err),
        ]
        .into_iter()
        .flatten()
        .next()
        .map(|e| view! { <div class="alert alert-error">{e.to_string()}</div> })
    };

    view! {
        <div class="space-y-6">
            <div class="breadcrumbs text-sm">
                <ul>
                    <li><A href="/prozesse">"Geschäftsprozesse"</A></li>
                    <li>
                        <Suspense fallback=|| "…">
                            {move || Suspend::new(async move {
                                process.await.map(|p| p.name).unwrap_or_default()
                            })}
                        </Suspense>
                    </li>
                </ul>
            </div>

            <Suspense fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match process.await {
                        Ok(p) => view! {
                            <div>
                                <h1 class="text-2xl font-bold">{p.name}</h1>
                                <p class="text-base-content/70">{p.description}</p>
                            </div>
                        }
                        .into_any(),
                        Err(e) => view! { <div class="alert alert-error">{e.to_string()}</div> }.into_any(),
                    }
                })}
            </Suspense>

            {error}

            <div class="card bg-base-100 shadow-sm">
                <div role="tablist" class="tabs tabs-border px-4 pt-2">
                    {[DetailTab::Description, DetailTab::Model]
                        .into_iter()
                        .map(|tab| {
                            view! {
                                <button
                                    role="tab"
                                    class="tab"
                                    class:tab-active=move || active_tab.get() == tab
                                    aria-selected=move || (active_tab.get() == tab).to_string()
                                    on:click=move |_| active_tab.set(tab)
                                >
                                    {tab.label()}
                                </button>
                            }
                        })
                        .collect_view()}
                </div>
                <div class="card-body pt-4" role="tabpanel">
                    <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                        {move || Suspend::new(async move {
                            let steps = steps.await.unwrap_or_default();
                            let name = process.await.map(|p| p.name).unwrap_or_default();
                            let content = match active_tab.get() {
                                DetailTab::Description => {
                                    let main = child_steps(&steps, None, None);
                                    view! {
                                        <Suggestions steps=steps.clone()/>
                                        <StepTable group=main.clone() prefix=String::new() actions form/>
                                        {branch_sections(&steps, &main, "", actions, form)}
                                    }
                                    .into_any()
                                }
                                DetailTab::Model => view! { <ProcessModel name steps=steps.clone()/> }.into_any(),
                            };
                            view! {
                                <BpmnCheck steps/>
                                {content}
                            }
                        })}
                    </Transition>
                </div>
            </div>

            <FormModal
                open=form.open
                title=Signal::derive(move || form.title())
                on_submit=save_step
                submit_disabled=Signal::derive(move || !form.draft.is_valid())
            >
                <Transition fallback=|| ()>
                    {move || Suspend::new(async move {
                        let own = process_id();
                        let others: Vec<Process> = processes
                            .await
                            .unwrap_or_default()
                            .into_iter()
                            .filter(|p| p.id != own)
                            .collect();
                        view! { <StepFormFields draft=form.draft processes=others/> }
                    })}
                </Transition>
            </FormModal>
        </div>
    }
}

/// Ergebnis der BPMN-Prüfung; bei Fehlern aufgeklappt.
#[component]
fn BpmnCheck(steps: Vec<ProcessStep>) -> impl IntoView {
    let findings = bpmn::check(&steps);
    if steps.is_empty() {
        return None;
    }
    let count = |level: Level| findings.iter().filter(|f| f.level == level).count();
    let (errors, warnings, hints) = (count(Level::Fehler), count(Level::Warnung), count(Level::Hinweis));

    let summary = if findings.is_empty() {
        "BPMN-Prüfung: keine Befunde".to_string()
    } else {
        let parts: Vec<String> = [(errors, "Fehler", "Fehler"), (warnings, "Warnung", "Warnungen"), (hints, "Hinweis", "Hinweise")]
            .into_iter()
            .filter(|(n, ..)| *n > 0)
            .map(|(n, one, many)| format!("{n} {}", if n == 1 { one } else { many }))
            .collect();
        format!("BPMN-Prüfung: {}", parts.join(", "))
    };
    let (icon, tone) = match (errors, warnings) {
        (0, 0) => ("✓", "border-success/40"),
        (0, _) => ("!", "border-warning/50"),
        _ => ("✕", "border-error/50"),
    };

    let items = findings
        .into_iter()
        .map(|f| {
            let badge = match f.level {
                Level::Fehler => "badge-error",
                Level::Warnung => "badge-warning",
                Level::Hinweis => "badge-info",
            };
            view! {
                <li class="flex items-start gap-2">
                    <span class=format!("badge badge-soft badge-sm shrink-0 {badge}")>{f.level.label()}</span>
                    <span>{f.message}</span>
                </li>
            }
        })
        .collect_view();

    Some(view! {
        <details class=format!("collapse collapse-arrow mb-4 border {tone} bg-base-100") open=errors > 0>
            <summary class="collapse-title text-sm font-semibold">{format!("{icon} {summary}")}</summary>
            <div class="collapse-content">
                <ul class="space-y-1 text-sm">{items}</ul>
            </div>
        </details>
    })
}

#[component]
fn StartBadge(event: String) -> impl IntoView {
    let text = if event.is_empty() { "▶ Start".to_string() } else { format!("▶ Start · {event}") };
    view! { <span class="badge badge-soft badge-success badge-sm" title="Startpunkt">{text}</span> }
}

#[component]
fn EndBadge(event: String) -> impl IntoView {
    let text = if event.is_empty() { "■ Ende".to_string() } else { format!("■ Ende · {event}") };
    view! { <span class="badge badge-soft badge-error badge-sm" title="Endpunkt">{text}</span> }
}

/// Vorschläge für Rolle und System aus allen bisher erfassten Schritten.
#[component]
fn Suggestions(steps: Vec<ProcessStep>) -> impl IntoView {
    let options = |pick: fn(&ProcessStep) -> &String| {
        let mut values: Vec<String> = steps
            .iter()
            .map(|s| pick(s).clone())
            .filter(|v| !v.is_empty())
            .collect();
        values.sort();
        values.dedup();
        values
            .into_iter()
            .map(|v| view! { <option value=v></option> })
            .collect_view()
    };

    view! {
        <datalist id="step-roles">{options(|s| &s.role)}</datalist>
        <datalist id="step-systems">{options(|s| &s.system)}</datalist>
    }
}

/// Für jeden Schritt einer Ebene mit einer Verzweigung: je eine Tabelle pro Zweig.
/// Zweige können selbst wieder Verzweigungen enthalten, daher rekursiv.
fn branch_sections(
    all: &[ProcessStep],
    group: &[ProcessStep],
    prefix: &str,
    actions: StepActions,
    form: StepForm,
) -> AnyView {
    group
        .iter()
        .enumerate()
        .filter(|(_, step)| has_branches(all, step))
        .map(|(i, step)| {
            view! {
                <GatewaySection
                    all=all.to_vec()
                    step=step.clone()
                    number=format!("{prefix}{}", i + 1)
                    actions
                    form
                />
            }
        })
        .collect_view()
        .into_any()
}

#[component]
fn GatewaySection(
    all: Vec<ProcessStep>,
    step: ProcessStep,
    number: String,
    actions: StepActions,
    form: StepForm,
) -> impl IntoView {
    let (kind, question) = if step.is_parallel() {
        (
            "Parallele Verzweigung",
            view! { <span>"Beide Zweige laufen gleichzeitig und werden wieder zusammengeführt."</span> }.into_any(),
        )
    } else if step.condition.trim().is_empty() {
        (
            "Exklusive Verzweigung",
            view! { <span class="text-warning">"Keine Frage erfasst – bitte im Schritt ergänzen."</span> }.into_any(),
        )
    } else {
        ("Exklusive Verzweigung", view! { <span>{step.condition.clone()}</span> }.into_any())
    };

    view! {
        <section class="mt-8 space-y-4 border-l-4 border-warning/40 pl-4">
            <div>
                <div class="text-xs text-base-content/60">
                    {format!("{kind} · Schritt {number}: {}", step.activity)}
                </div>
                <div class="font-semibold">{question}</div>
            </div>
            <BranchTable all=all.clone() parent=step.clone() branch="Ja" number=number.clone() actions form/>
            <BranchTable all parent=step branch="Nein" number actions form/>
        </section>
    }
}

#[component]
fn BranchTable(
    all: Vec<ProcessStep>,
    parent: ProcessStep,
    branch: &'static str,
    number: String,
    actions: StepActions,
    form: StepForm,
) -> impl IntoView {
    let parallel = parent.is_parallel();
    let label = branch_label(parallel, branch);
    let group = child_steps(&all, Some(parent.id), Some(branch));
    let prefix = if parallel {
        format!("{number}.{}", if branch == "Ja" { "P1." } else { "P2." })
    } else {
        format!("{number}.{}", &branch[..1])
    };
    let (badge, border, text) = match (parallel, branch) {
        (true, _) => ("badge-info", "border-info/40", "Schritte, die parallel ausgeführt werden".to_string()),
        (false, "Ja") => ("badge-success", "border-success/40", format!("Schritte, wenn die Antwort „{label}“ ist")),
        (false, _) => ("badge-error", "border-error/40", format!("Schritte, wenn die Antwort „{label}“ ist")),
    };

    view! {
        <div class=format!("rounded-box border {border}")>
            <div class="flex items-center gap-2 border-b border-base-300 bg-base-200/50 px-4 py-2">
                <span class=format!("badge badge-soft {badge}")>{label}</span>
                <span class="text-sm text-base-content/70">{text}</span>
            </div>
            <div class="p-2">
                <StepTable
                    group=group.clone()
                    prefix=prefix.clone()
                    actions
                    form
                    parent_step_id=parent.id
                    branch=branch
                    label=label
                />
            </div>
            <div class="px-4 pb-4">{branch_sections(&all, &group, &prefix, actions, form)}</div>
        </div>
    }
}

#[component]
fn StepTable(
    group: Vec<ProcessStep>,
    /// Nummer des übergeordneten Schritts, z. B. "2.J" für den Ja-Zweig von Schritt 2.
    prefix: String,
    actions: StepActions,
    form: StepForm,
    /// Schritt mit der Verzweigung, zu deren Zweig die Tabelle gehört; leer im Hauptablauf.
    #[prop(optional)]
    parent_step_id: Option<i64>,
    #[prop(optional)] branch: Option<&'static str>,
    /// Anzeigename des Zweigs, z. B. "Ja" oder "Zweig 1".
    #[prop(optional)]
    label: Option<&'static str>,
) -> impl IntoView {
    let count = group.len();
    let rows = group
        .into_iter()
        .enumerate()
        .map(|(index, step)| {
            view! {
                <StepRow
                    step
                    number=format!("{prefix}{}", index + 1)
                    is_first=index == 0
                    is_last=index + 1 == count
                    actions
                    form
                />
            }
        })
        .collect_view();

    view! {
        <div class="overflow-x-auto">
            <table class="table table-sm">
                <thead>
                    <tr>
                        <th class="w-12">"#"</th>
                        <th class="w-24">"Start/Ende"</th>
                        <th>"Element"</th>
                        <th>"Bezeichnung"</th>
                        <th>"Rolle"</th>
                        <th>"Input"</th>
                        <th>"Output"</th>
                        <th>"Verzweigung"</th>
                        <th>"System"</th>
                        <th class="w-28"></th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>
        <button class="btn btn-ghost btn-sm mt-2" on:click=move |_| form.new_step(parent_step_id, branch, label)>
            "+ Schritt hinzufügen"
        </button>
    }
}

/// Auswahlfeld für einen der festen Werte; `empty` ist die Beschriftung des leeren Werts.
#[component]
fn Choice(
    value: RwSignal<String>,
    label: &'static str,
    options: &'static [&'static str],
    #[prop(optional)] empty: Option<&'static str>,
) -> impl IntoView {
    view! {
        <label class="floating-label">
            <span>{label}</span>
            <select
                class="select w-full"
                prop:value=move || value.get()
                on:change=move |ev| value.set(event_target_value(&ev))
            >
                {empty.is_none().then(|| view! { <option value="" disabled>"Bitte wählen"</option> })}
                {options
                    .iter()
                    .map(|&option| {
                        let text = if option.is_empty() { empty.unwrap_or("–") } else { option };
                        view! { <option value=option selected=move || value.get() == option>{text}</option> }
                    })
                    .collect_view()}
            </select>
        </label>
    }
}

/// Die Felder im Schritt-Modalfenster.
#[component]
fn StepFormFields(draft: StepDraft, processes: Vec<Process>) -> impl IntoView {
    let field = move |value: RwSignal<String>, label: &'static str, list: Option<&'static str>| {
        view! {
            <label class="floating-label">
                <span>{label}</span>
                <input class="input w-full" placeholder=label list=list bind:value=value/>
            </label>
        }
    };
    let element_is = move |element: &'static str| move || draft.element.get() == element;

    let process_options = processes
        .into_iter()
        .map(|p| {
            let id = p.id;
            view! {
                <option value=id.to_string() selected=move || draft.called_process_id.get() == Some(id)>
                    {p.name}
                </option>
            }
        })
        .collect_view();

    view! {
        <label class="floating-label">
            <span>"Bezeichnung"</span>
            <input
                class="input w-full"
                placeholder="Bezeichnung (z. B. „Rechnung prüfen“)"
                required
                bind:value=draft.activity
            />
        </label>
        <div class="grid gap-4 sm:grid-cols-2">
            <Choice value=draft.element label="Element" options=ELEMENTS/>
            <div class:hidden=move || !element_is("Aufgabe")()>
                <Choice value=draft.task_type label="Aufgabentyp" options=TASK_TYPES empty="Ohne Typ"/>
            </div>
            <div class:hidden=move || !element_is("Zwischenereignis")()>
                <Choice value=draft.event_type label="Ereignistyp" options=EVENT_TYPES/>
            </div>
            <label class="floating-label" class:hidden=move || !element_is("Teilprozess")()>
                <span>"Aufgerufener Prozess"</span>
                <select
                    class="select w-full"
                    prop:value=move || draft.called_process_id.get().map(|id| id.to_string()).unwrap_or_default()
                    on:change=move |ev| draft.called_process_id.set(event_target_value(&ev).parse().ok())
                >
                    <option value="" selected=move || draft.called_process_id.get().is_none()>"Keiner (eingebettet)"</option>
                    {process_options}
                </select>
            </label>
        </div>
        <div class="grid gap-4 sm:grid-cols-2">
            {field(draft.role, "Rolle (Bahn)", Some("step-roles"))}
            {field(draft.system, "System", Some("step-systems"))}
            {field(draft.input, "Input (Datenobjekt)", None)}
            {field(draft.output, "Output (Datenobjekt)", None)}
            <div class:hidden=element_is("Zwischenereignis")>
                <Choice value=draft.loop_type label="Wiederholung" options=LOOP_TYPES empty="Keine"/>
            </div>
            <Choice value=draft.gateway label="Verzweigung danach" options=GATEWAYS empty="Keine"/>
        </div>
        <div class:hidden=move || draft.gateway.get() != "Exklusiv">
            {field(draft.condition, "Frage der Verzweigung (z. B. „Betrag > 1000?“)", None)}
        </div>
        <div class="space-y-3">
            <label class="flex cursor-pointer items-center gap-3">
                <input type="checkbox" class="checkbox checkbox-success checkbox-sm" bind:checked=draft.is_start/>
                <span>
                    "Startpunkt"
                    <span class="block text-xs text-base-content/60">
                        "Einstieg in den Prozess. Ein anderer Startpunkt wird dabei aufgehoben."
                    </span>
                </span>
            </label>
            <div class="grid gap-4 pl-8 sm:grid-cols-2" class:hidden=move || !draft.is_start.get()>
                <Choice value=draft.start_event label="Auslöser" options=START_EVENTS empty="Ohne (Blanko)"/>
                {field(draft.start_label, "Startereignis (z. B. „Antrag eingegangen“)", None)}
            </div>
            <label class="flex cursor-pointer items-center gap-3">
                <input type="checkbox" class="checkbox checkbox-error checkbox-sm" bind:checked=draft.is_end/>
                <span>
                    "Endpunkt"
                    <span class="block text-xs text-base-content/60">
                        "Der Prozess endet nach diesem Schritt (bei einer Verzweigung nach der Zusammenführung)."
                    </span>
                </span>
            </label>
            <div class="grid gap-4 pl-8 sm:grid-cols-2" class:hidden=move || !draft.is_end.get()>
                <Choice value=draft.end_event label="Ergebnis" options=END_EVENTS empty="Ohne (Blanko)"/>
                {field(draft.end_label, "Endereignis (z. B. „Antrag abgelehnt“)", None)}
            </div>
        </div>
    }
}

/// Element mit Typ und Wiederholung für die Tabelle, z. B. "Aufgabe · Benutzer ↻".
fn element_text(step: &ProcessStep) -> String {
    let mut text = match step.element.as_str() {
        "Zwischenereignis" => format!("◎ {}", step.event_type),
        "Teilprozess" => match &step.called_process_name {
            Some(name) => format!("⊞ Teilprozess → {name}"),
            None => "⊞ Teilprozess".to_string(),
        },
        _ if step.task_type.is_empty() => "▭ Aufgabe".to_string(),
        _ => format!("▭ {}", step.task_type),
    };
    let marker = match step.loop_type.as_str() {
        "Schleife" => " ↻",
        "Mehrfach parallel" => " ⦀",
        "Mehrfach sequenziell" => " ☰",
        _ => "",
    };
    text.push_str(marker);
    text
}

#[component]
fn StepRow(
    step: ProcessStep,
    number: String,
    is_first: bool,
    is_last: bool,
    actions: StepActions,
    form: StepForm,
) -> impl IntoView {
    let id = step.id;
    let stored = StoredValue::new(step.clone());
    let edit = move || stored.with_value(|step| form.edit(step));
    let element = element_text(&step);
    let gateway = match step.gateway.as_str() {
        "Parallel" => view! { <span>"+ parallel"</span> }.into_any(),
        "Exklusiv" => view! { <span class="italic">{format!("✕ {}", step.condition)}</span> }.into_any(),
        _ => ().into_any(),
    };

    view! {
        <tr
            class="hover:bg-base-200 cursor-pointer select-none"
            title="Doppelklick zum Bearbeiten"
            on:dblclick=move |_| edit()
        >
            <td class="text-base-content/60 whitespace-nowrap">{number}</td>
            <td class="whitespace-nowrap">
                <div class="flex flex-col items-start gap-1">
                    {step.is_start.then(|| view! { <StartBadge event=step.start_event.clone()/> })}
                    {step.is_end.then(|| view! { <EndBadge event=step.end_event.clone()/> })}
                </div>
            </td>
            <td class="whitespace-nowrap text-base-content/70">{element}</td>
            <td class="font-medium">{step.activity}</td>
            <td><RoleBadge role=step.role/></td>
            <td>{step.input}</td>
            <td>{step.output}</td>
            <td>{gateway}</td>
            <td>{step.system}</td>
            <td class="text-right whitespace-nowrap">
                <button class="btn btn-ghost btn-xs" title="Nach oben" disabled=is_first
                    on:click=move |_| { actions.move_step.dispatch(MoveStep { id, up: true }); }>"▲"</button>
                <button class="btn btn-ghost btn-xs" title="Nach unten" disabled=is_last
                    on:click=move |_| { actions.move_step.dispatch(MoveStep { id, up: false }); }>"▼"</button>
                <button class="btn btn-ghost btn-xs" title="Bearbeiten" on:click=move |_| edit()>"✎"</button>
                <button class="btn btn-ghost btn-xs text-error" title="Löschen"
                    on:click=move |_| { actions.delete.dispatch(DeleteStep { id }); }>"✕"</button>
            </td>
        </tr>
    }
}

#[component]
fn RoleBadge(role: String) -> impl IntoView {
    (!role.is_empty()).then(|| view! { <span class="badge badge-soft badge-primary">{role}</span> })
}
