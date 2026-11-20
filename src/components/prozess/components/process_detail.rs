use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_params_map};

use super::process_model::ProcessModel;
use crate::backend::prozess::model::{ProcessStep, StepFields, child_steps};
use crate::backend::prozess::server_fns::{
    AddStep, DeleteStep, MoveStep, UpdateStep, get_process, list_steps,
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
}

impl StepDraft {
    fn new(fields: StepFields) -> Self {
        Self {
            activity: RwSignal::new(fields.activity),
            role: RwSignal::new(fields.role),
            input: RwSignal::new(fields.input),
            output: RwSignal::new(fields.output),
            condition: RwSignal::new(fields.condition),
            system: RwSignal::new(fields.system),
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
        }
    }

    fn set(&self, fields: StepFields) {
        self.activity.set(fields.activity);
        self.role.set(fields.role);
        self.input.set(fields.input);
        self.output.set(fields.output);
        self.condition.set(fields.condition);
        self.system.set(fields.system);
    }
}

/// Wohin ein Schritt aus dem Modalfenster gespeichert wird.
#[derive(Clone, PartialEq)]
enum StepTarget {
    /// Neuer Schritt im Hauptablauf (`parent_step_id` leer) oder im Ja-/Nein-Zweig einer Bedingung.
    New { parent_step_id: Option<i64>, branch: Option<String> },
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
            draft: StepDraft::new(StepFields::default()),
            open: RwSignal::new(false),
            target: RwSignal::new(StepTarget::New { parent_step_id: None, branch: None }),
        }
    }

    fn new_step(&self, parent_step_id: Option<i64>, branch: Option<&str>) {
        self.draft.set(StepFields::default());
        self.target.set(StepTarget::New { parent_step_id, branch: branch.map(str::to_string) });
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
            StepTarget::New { branch: Some(branch), .. } => format!("Neuer Schritt im Zweig „{branch}“"),
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
        let fields = form.draft.fields();
        if fields.activity.trim().is_empty() {
            return;
        }
        match form.target.get_untracked() {
            StepTarget::Edit(id) => {
                actions.update.dispatch(UpdateStep { id, fields });
            }
            StepTarget::New { parent_step_id, branch } => {
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
                            match active_tab.get() {
                                DetailTab::Description => {
                                    let main = child_steps(&steps, None, None);
                                    view! {
                                        <Suggestions steps=steps.clone()/>
                                        <StepTable group=main.clone() prefix=String::new() actions form/>
                                        {condition_sections(&steps, &main, "", actions, form)}
                                    }
                                    .into_any()
                                }
                                DetailTab::Model => view! { <ProcessModel steps/> }.into_any(),
                            }
                        })}
                    </Transition>
                </div>
            </div>

            <FormModal
                open=form.open
                title=Signal::derive(move || form.title())
                on_submit=save_step
                submit_disabled=Signal::derive(move || form.draft.activity.get().trim().is_empty())
            >
                <StepFormFields draft=form.draft/>
            </FormModal>
        </div>
    }
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

/// Für jeden Schritt einer Ebene mit einer Frage in "Bedingung": je eine Tabelle für Ja und Nein.
/// Zweige können selbst wieder Bedingungen enthalten, daher rekursiv.
fn condition_sections(
    all: &[ProcessStep],
    group: &[ProcessStep],
    prefix: &str,
    actions: StepActions,
    form: StepForm,
) -> AnyView {
    group
        .iter()
        .enumerate()
        .filter(|(_, step)| {
            !step.condition.trim().is_empty()
                || all.iter().any(|s| s.parent_step_id == Some(step.id))
        })
        .map(|(i, step)| {
            view! {
                <ConditionSection
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
fn ConditionSection(
    all: Vec<ProcessStep>,
    step: ProcessStep,
    number: String,
    actions: StepActions,
    form: StepForm,
) -> impl IntoView {
    let question = if step.condition.trim().is_empty() {
        view! { <span class="text-warning">"Keine Frage erfasst – die Zweige bleiben gespeichert."</span> }
            .into_any()
    } else {
        view! { <span>{step.condition.clone()}</span> }.into_any()
    };

    view! {
        <section class="mt-8 space-y-4 border-l-4 border-warning/40 pl-4">
            <div>
                <div class="text-xs text-base-content/60">
                    {format!("Bedingung · Schritt {number}: {}", step.activity)}
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
    let group = child_steps(&all, Some(parent.id), Some(branch));
    let prefix = format!("{number}.{}", &branch[..1]);
    let (badge, border) = if branch == "Ja" {
        ("badge-success", "border-success/40")
    } else {
        ("badge-error", "border-error/40")
    };

    view! {
        <div class=format!("rounded-box border {border}")>
            <div class="flex items-center gap-2 border-b border-base-300 bg-base-200/50 px-4 py-2">
                <span class=format!("badge badge-soft {badge}")>{branch}</span>
                <span class="text-sm text-base-content/70">
                    {format!("Schritte, wenn die Antwort „{branch}“ ist")}
                </span>
            </div>
            <div class="p-2">
                <StepTable
                    group=group.clone()
                    prefix=prefix.clone()
                    actions
                    form
                    parent_step_id=parent.id
                    branch=branch
                />
            </div>
            <div class="px-4 pb-4">{condition_sections(&all, &group, &prefix, actions, form)}</div>
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
    /// Schritt mit der Bedingung, zu dessen Zweig die Tabelle gehört; leer im Hauptablauf.
    #[prop(optional)]
    parent_step_id: Option<i64>,
    #[prop(optional)] branch: Option<&'static str>,
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
                        <th>"Aktivität"</th>
                        <th>"Rolle"</th>
                        <th>"Input"</th>
                        <th>"Output"</th>
                        <th>"Bedingung"</th>
                        <th>"System"</th>
                        <th class="w-28"></th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>
        <button class="btn btn-ghost btn-sm mt-2" on:click=move |_| form.new_step(parent_step_id, branch)>
            "+ Schritt hinzufügen"
        </button>
    }
}

/// Die Felder im Schritt-Modalfenster.
#[component]
fn StepFormFields(draft: StepDraft) -> impl IntoView {
    let field = move |value: RwSignal<String>, label: &'static str, list: Option<&'static str>| {
        view! {
            <label class="floating-label">
                <span>{label}</span>
                <input class="input w-full" placeholder=label list=list bind:value=value/>
            </label>
        }
    };

    view! {
        <label class="floating-label">
            <span>"Aktivität"</span>
            <input class="input w-full" placeholder="Aktivität" required bind:value=draft.activity/>
        </label>
        <div class="grid gap-4 sm:grid-cols-2">
            {field(draft.role, "Rolle", Some("step-roles"))}
            {field(draft.system, "System", Some("step-systems"))}
            {field(draft.input, "Input", None)}
            {field(draft.output, "Output", None)}
        </div>
        {field(draft.condition, "Bedingung (Frage, z. B. „Betrag > 1000?“)", None)}
    }
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

    view! {
        <tr
            class="hover:bg-base-200 cursor-pointer select-none"
            title="Doppelklick zum Bearbeiten"
            on:dblclick=move |_| edit()
        >
            <td class="text-base-content/60 whitespace-nowrap">{number}</td>
            <td class="font-medium">{step.activity}</td>
            <td><RoleBadge role=step.role/></td>
            <td>{step.input}</td>
            <td>{step.output}</td>
            <td class="italic">{step.condition}</td>
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
