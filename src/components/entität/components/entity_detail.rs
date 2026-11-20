use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_params_map};

use crate::backend::entitaet::model::{Entity, EntityColumn};
use crate::backend::entitaet::server_fns::{
    AddEntityColumn, DeleteEntityColumn, RenameEntity, UpdateEntityColumn, get_entity,
    list_entities, list_entity_columns,
};
use crate::components::modal::FormModal;

/// Häufige Datentypen als Vorschlag; eigene Typen sind ebenfalls erlaubt.
const DATA_TYPES: &[&str] = &[
    "Text", "Ganzzahl", "Dezimalzahl", "Betrag", "Datum", "Datum/Zeit", "Ja/Nein", "Referenz",
];

#[derive(Clone, Copy)]
struct ColumnActions {
    add: ServerAction<AddEntityColumn>,
    update: ServerAction<UpdateEntityColumn>,
    delete: ServerAction<DeleteEntityColumn>,
}

/// Eingabefelder des Spalten-Modalfensters, für neue und bearbeitete Spalten.
#[derive(Clone, Copy)]
struct ColumnDraft {
    name: RwSignal<String>,
    data_type: RwSignal<String>,
    meaning: RwSignal<String>,
    /// Id der referenzierten Entität als Text (Wert der Auswahlliste); leer = kein Verweis.
    reference: RwSignal<String>,
}

impl ColumnDraft {
    fn new(column: Option<&EntityColumn>) -> Self {
        let text = |pick: fn(&EntityColumn) -> &String| column.map(pick).cloned().unwrap_or_default();
        Self {
            name: RwSignal::new(text(|c| &c.name)),
            data_type: RwSignal::new(text(|c| &c.data_type)),
            meaning: RwSignal::new(text(|c| &c.meaning)),
            reference: RwSignal::new(
                column
                    .and_then(|c| c.references_entity_id)
                    .map(|id| id.to_string())
                    .unwrap_or_default(),
            ),
        }
    }

    fn reference_id(&self) -> Option<i64> {
        self.reference.get_untracked().parse().ok()
    }

    fn reset(&self, column: Option<&EntityColumn>) {
        let fresh = Self::new(column);
        self.name.set(fresh.name.get_untracked());
        self.data_type.set(fresh.data_type.get_untracked());
        self.meaning.set(fresh.meaning.get_untracked());
        self.reference.set(fresh.reference.get_untracked());
    }
}

#[component]
pub fn EntityDetailPage() -> impl IntoView {
    let params = use_params_map();
    let entity_id = move || {
        params
            .read()
            .get("id")
            .and_then(|id| id.parse::<i64>().ok())
            .unwrap_or_default()
    };

    let rename = ServerAction::<RenameEntity>::new();
    let actions = ColumnActions {
        add: ServerAction::new(),
        update: ServerAction::new(),
        delete: ServerAction::new(),
    };

    let entity = Resource::new(move || (entity_id(), rename.version().get()), |(id, _)| get_entity(id));
    let entities = Resource::new(move || rename.version().get(), |_| list_entities());
    let columns = Resource::new(
        move || {
            (
                entity_id(),
                actions.add.version().get(),
                actions.update.version().get(),
                actions.delete.version().get(),
            )
        },
        |(id, ..)| list_entity_columns(id),
    );

    // Ein Modalfenster für neue und bearbeitete Spalten.
    let draft = ColumnDraft::new(None);
    let modal_open = RwSignal::new(false);
    let editing_id = RwSignal::new(None::<i64>);
    let open_column = Callback::new(move |column: Option<EntityColumn>| {
        draft.reset(column.as_ref());
        editing_id.set(column.map(|c| c.id));
        modal_open.set(true);
    });
    let save_column = move || {
        if draft.name.get_untracked().trim().is_empty() {
            return;
        }
        match editing_id.get_untracked() {
            Some(id) => {
                actions.update.dispatch(UpdateEntityColumn {
                    id,
                    name: draft.name.get_untracked(),
                    data_type: draft.data_type.get_untracked(),
                    meaning: draft.meaning.get_untracked(),
                    references_entity_id: draft.reference_id(),
                });
            }
            None => {
                actions.add.dispatch(AddEntityColumn {
                    entity_id: entity_id(),
                    name: draft.name.get_untracked(),
                    data_type: draft.data_type.get_untracked(),
                    meaning: draft.meaning.get_untracked(),
                    references_entity_id: draft.reference_id(),
                });
            }
        }
        modal_open.set(false);
    };
    let modal_title = Signal::derive(move || {
        if editing_id.get().is_some() { "Spalte bearbeiten" } else { "Neue Spalte" }.to_string()
    });

    let error = move || {
        [
            rename.value().get().and_then(Result::err),
            actions.add.value().get().and_then(Result::err),
            actions.update.value().get().and_then(Result::err),
            actions.delete.value().get().and_then(Result::err),
        ]
        .into_iter()
        .flatten()
        .next()
        .map(|e| view! { <div class="alert alert-error">{e.to_string()}</div> })
    };

    view! {
        <div class="mx-auto max-w-5xl space-y-6">
            <div class="breadcrumbs text-sm">
                <ul>
                    <li><A href="/entitaeten">"Entitäten"</A></li>
                    <li>
                        <Suspense fallback=|| "…">
                            {move || Suspend::new(async move {
                                entity.await.map(|e| e.name).unwrap_or_default()
                            })}
                        </Suspense>
                    </li>
                </ul>
            </div>

            <Suspense fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match entity.await {
                        Ok(e) => view! { <EntityHeader id=e.id name=e.name rename/> }.into_any(),
                        Err(e) => view! { <div class="alert alert-error">{e.to_string()}</div> }.into_any(),
                    }
                })}
            </Suspense>

            {error}

            <datalist id="column-types">
                {DATA_TYPES.iter().map(|t| view! { <option value=*t></option> }).collect_view()}
            </datalist>

            <div class="card bg-base-100 shadow-sm">
                <div class="card-body">
                    <div class="flex items-center justify-between gap-4">
                        <h2 class="card-title">"Spalten"</h2>
                        <button class="btn btn-primary btn-sm" on:click=move |_| open_column.run(None)>
                            "+ Spalte hinzufügen"
                        </button>
                    </div>
                    <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                        {move || Suspend::new(async move {
                            let columns = columns.await.unwrap_or_default();
                            let entities = entities.await.unwrap_or_default();
                            view! { <ColumnTable columns entities actions on_edit=open_column/> }
                        })}
                    </Transition>
                </div>
            </div>

            <FormModal
                open=modal_open
                title=modal_title
                on_submit=save_column
                submit_disabled=Signal::derive(move || draft.name.get().trim().is_empty())
            >
                <ColumnFields draft entities/>
            </FormModal>
        </div>
    }
}

/// Bezeichnung der Entität; Umbenennen im Modalfenster.
#[component]
fn EntityHeader(id: i64, name: String, rename: ServerAction<RenameEntity>) -> impl IntoView {
    let open = RwSignal::new(false);
    let value = RwSignal::new(name.clone());
    let original = StoredValue::new(name.clone());
    let open_modal = move |_| {
        value.set(original.get_value());
        open.set(true);
    };
    let save = move || {
        rename.dispatch(RenameEntity { id, name: value.get_untracked() });
        open.set(false);
    };

    view! {
        <div class="flex items-center gap-2">
            <h1 class="text-2xl font-bold">{name}</h1>
            <button class="btn btn-ghost btn-sm" title="Umbenennen" on:click=open_modal>"✎"</button>
        </div>
        <FormModal
            open
            title="Entität umbenennen".to_string()
            on_submit=save
            submit_disabled=Signal::derive(move || value.get().trim().is_empty())
        >
            <label class="floating-label">
                <span>"Bezeichnung"</span>
                <input class="input w-full" placeholder="Bezeichnung" required bind:value=value/>
            </label>
        </FormModal>
    }
}

#[component]
fn ColumnTable(
    columns: Vec<EntityColumn>,
    entities: Vec<Entity>,
    actions: ColumnActions,
    on_edit: Callback<Option<EntityColumn>>,
) -> impl IntoView {
    if columns.is_empty() {
        return view! { <p class="text-base-content/60">"Noch keine Spalten erfasst."</p> }.into_any();
    }
    let rows = columns
        .into_iter()
        .enumerate()
        .map(|(index, column)| {
            let target = column
                .references_entity_id
                .and_then(|id| entities.iter().find(|e| e.id == id).cloned());
            view! { <ColumnRow column number=index + 1 target actions on_edit/> }
        })
        .collect_view();

    view! {
        <div class="overflow-x-auto">
            <table class="table table-sm">
                <thead>
                    <tr>
                        <th class="w-8">"#"</th>
                        <th>"Bezeichnung"</th>
                        <th>"Typ"</th>
                        <th>"Bedeutung"</th>
                        <th>"Verweist auf"</th>
                        <th class="w-24"></th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>
    }
    .into_any()
}

#[component]
fn ColumnRow(
    column: EntityColumn,
    number: usize,
    /// Entität, auf die die Spalte verweist.
    target: Option<Entity>,
    actions: ColumnActions,
    on_edit: Callback<Option<EntityColumn>>,
) -> impl IntoView {
    let id = column.id;
    let stored = StoredValue::new(column.clone());
    let edit = move || on_edit.run(Some(stored.get_value()));

    view! {
        <tr
            class="hover:bg-base-200 cursor-pointer select-none"
            title="Doppelklick zum Bearbeiten"
            on:dblclick=move |_| edit()
        >
            <td class="text-base-content/60">{number}</td>
            <td class="font-medium">{column.name}</td>
            <td>
                {(!column.data_type.is_empty())
                    .then(|| view! { <span class="badge badge-soft badge-secondary">{column.data_type}</span> })}
            </td>
            <td class="text-base-content/80">{column.meaning}</td>
            <td>
                {target.map(|e| view! {
                    <A href=format!("/entitaeten/{}", e.id) attr:class="link link-hover">{format!("→ {}", e.name)}</A>
                })}
            </td>
            <td class="text-right whitespace-nowrap">
                <button class="btn btn-ghost btn-xs" title="Bearbeiten" on:click=move |_| edit()>"✎"</button>
                <button class="btn btn-ghost btn-xs text-error" title="Löschen"
                    on:click=move |_| { actions.delete.dispatch(DeleteEntityColumn { id }); }>"✕"</button>
            </td>
        </tr>
    }
}

/// Die Felder im Spalten-Modalfenster.
#[component]
fn ColumnFields(
    draft: ColumnDraft,
    entities: Resource<Result<Vec<Entity>, ServerFnError>>,
) -> impl IntoView {
    // Wer einen Verweis wählt, meint in der Regel den Typ "Referenz".
    let on_reference = move |_| {
        if !draft.reference.get_untracked().is_empty() && draft.data_type.get_untracked().is_empty() {
            draft.data_type.set("Referenz".to_string());
        }
    };

    view! {
        <label class="floating-label">
            <span>"Bezeichnung"</span>
            <input class="input w-full" placeholder="Bezeichnung" required bind:value=draft.name/>
        </label>
        <label class="floating-label">
            <span>"Typ"</span>
            <input class="input w-full" placeholder="Typ" list="column-types" bind:value=draft.data_type/>
        </label>
        <label class="floating-label">
            <span>"Bedeutung"</span>
            <textarea class="textarea w-full" placeholder="Bedeutung" bind:value=draft.meaning></textarea>
        </label>
        <label class="floating-label">
            <span>"Verweist auf"</span>
            <select class="select w-full" bind:value=draft.reference on:change=on_reference>
                <option value="">"— kein Verweis —"</option>
                <Transition>
                    {move || Suspend::new(async move {
                        entities
                            .await
                            .unwrap_or_default()
                            .into_iter()
                            .map(|e| {
                                let value = e.id.to_string();
                                let selected = {
                                    let value = value.clone();
                                    move || draft.reference.get() == value
                                };
                                view! { <option value=value selected=selected>{e.name}</option> }
                            })
                            .collect_view()
                    })}
                </Transition>
            </select>
        </label>
    }
}
