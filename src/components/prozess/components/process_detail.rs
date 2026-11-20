use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_params_map};

use super::process_model::ProcessModel;
use crate::backend::prozess::model::{ProcessStep, StepFields, child_steps};
use crate::backend::prozess::server_fns::{
    AddStep, DeleteStep, MoveStep, UpdateStep, get_process, list_steps,
};

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

/// Eingabefelder einer Tabellenzeile, sowohl für neue als auch für bearbeitete Schritte.
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

    let draft = StepDraft::new(StepFields::default());
    let add_step = move || {
        if draft.activity.get_untracked().trim().is_empty() {
            return;
        }
        actions.add.dispatch(AddStep {
            process_id: process_id(),
            parent_step_id: None,
            branch: None,
            fields: draft.fields(),
        });
        draft.set(StepFields::default());
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
                                        <StepTable group=main.clone() prefix=String::new() actions draft on_add=add_step/>
                                        {condition_sections(&steps, &main, "", actions)}
                                    }
                                    .into_any()
                                }
                                DetailTab::Model => view! { <ProcessModel steps/> }.into_any(),
                            }
                        })}
                    </Transition>
                </div>
            </div>
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
            <BranchTable all=all.clone() parent=step.clone() branch="Ja" number=number.clone() actions/>
            <BranchTable all parent=step branch="Nein" number actions/>
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
) -> impl IntoView {
    let group = child_steps(&all, Some(parent.id), Some(branch));
    let prefix = format!("{number}.{}", &branch[..1]);
    let (badge, border) = if branch == "Ja" {
        ("badge-success", "border-success/40")
    } else {
        ("badge-error", "border-error/40")
    };

    let draft = StepDraft::new(StepFields::default());
    let (process_id, parent_id) = (parent.process_id, parent.id);
    let on_add = move || {
        if draft.activity.get_untracked().trim().is_empty() {
            return;
        }
        actions.add.dispatch(AddStep {
            process_id,
            parent_step_id: Some(parent_id),
            branch: Some(branch.to_string()),
            fields: draft.fields(),
        });
        draft.set(StepFields::default());
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
                <StepTable group=group.clone() prefix=format!("{prefix}") actions draft on_add/>
            </div>
            <div class="px-4 pb-4">{condition_sections(&all, &group, &prefix, actions)}</div>
        </div>
    }
}

#[component]
fn StepTable(
    group: Vec<ProcessStep>,
    /// Nummer des übergeordneten Schritts, z. B. "2.J" für den Ja-Zweig von Schritt 2.
    prefix: String,
    actions: StepActions,
    draft: StepDraft,
    on_add: impl Fn() + Copy + Send + Sync + 'static,
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
                <tfoot>
                    <tr class="bg-base-200/50">
                        <td>"+"</td>
                        <DraftCells draft on_enter=on_add/>
                        <td class="text-right">
                            <button
                                class="btn btn-primary btn-sm"
                                disabled=move || draft.activity.get().trim().is_empty()
                                on:click=move |_| on_add()
                            >
                                "Hinzufügen"
                            </button>
                        </td>
                    </tr>
                </tfoot>
            </table>
        </div>
    }
}

/// Die Eingabefelder einer Zeile; Enter speichert.
#[component]
fn DraftCells(draft: StepDraft, on_enter: impl Fn() + Copy + Send + Sync + 'static) -> impl IntoView {
    let on_keydown = move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Enter" {
            on_enter();
        }
    };
    let field = move |value: RwSignal<String>,
                      placeholder: &'static str,
                      list: Option<&'static str>,
                      width: &'static str| {
        view! {
            <td>
                <input
                    class=format!("input input-sm w-full {width}")
                    placeholder=placeholder
                    list=list
                    bind:value=value
                    on:keydown=on_keydown
                />
            </td>
        }
    };

    view! {
        {field(draft.activity, "Aktivität", None, "min-w-28")}
        {field(draft.role, "Rolle", Some("step-roles"), "min-w-24")}
        {field(draft.input, "Input", None, "min-w-24")}
        {field(draft.output, "Output", None, "min-w-24")}
        {field(draft.condition, "Frage?", None, "min-w-32")}
        {field(draft.system, "System", Some("step-systems"), "min-w-24")}
    }
}

#[component]
fn StepRow(
    step: ProcessStep,
    number: String,
    is_first: bool,
    is_last: bool,
    actions: StepActions,
) -> impl IntoView {
    let editing = RwSignal::new(false);
    let draft = StepDraft::new(step.fields());
    let original = StoredValue::new(step.fields());
    let id = step.id;

    let save = move || {
        if draft.activity.get_untracked().trim().is_empty() {
            return;
        }
        actions.update.dispatch(UpdateStep { id, fields: draft.fields() });
        editing.set(false);
    };
    let cancel = move || {
        draft.set(original.get_value());
        editing.set(false);
    };

    let display = move || {
        let step = step.clone();
        view! {
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
                <button class="btn btn-ghost btn-xs" title="Bearbeiten" on:click=move |_| editing.set(true)>"✎"</button>
                <button class="btn btn-ghost btn-xs text-error" title="Löschen"
                    on:click=move |_| { actions.delete.dispatch(DeleteStep { id }); }>"✕"</button>
            </td>
        }
    };

    view! {
        <tr
            class="hover:bg-base-200"
            class:cursor-pointer=move || !editing.get()
            title=move || (!editing.get()).then_some("Doppelklick zum Bearbeiten")
            on:dblclick=move |_| editing.set(true)
        >
            <td class="text-base-content/60 whitespace-nowrap">{number}</td>
            <Show when=move || editing.get() fallback=display>
                <DraftCells draft on_enter=save/>
                <td class="text-right whitespace-nowrap">
                    <button class="btn btn-primary btn-xs" on:click=move |_| save()>"Speichern"</button>
                    <button class="btn btn-ghost btn-xs" on:click=move |_| cancel()>"Abbrechen"</button>
                </td>
            </Show>
        </tr>
    }
}

#[component]
fn RoleBadge(role: String) -> impl IntoView {
    (!role.is_empty()).then(|| view! { <span class="badge badge-soft badge-primary">{role}</span> })
}
