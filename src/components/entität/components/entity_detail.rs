use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_params_map};

use crate::backend::entitaet::model::{Entity, EntityColumn};
use crate::backend::entitaet::server_fns::{
    AddEntityColumn, DeleteEntityColumn, RenameEntity, UpdateEntityColumn, get_entity,
    list_entities, list_entity_columns,
};

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

/// Eingabefelder einer Spalten-Zeile, für neue und bearbeitete Spalten.
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

    let draft = ColumnDraft::new(None);
    let add_column = move || {
        if draft.name.get_untracked().trim().is_empty() {
            return;
        }
        actions.add.dispatch(AddEntityColumn {
            entity_id: entity_id(),
            name: draft.name.get_untracked(),
            data_type: draft.data_type.get_untracked(),
            meaning: draft.meaning.get_untracked(),
            references_entity_id: draft.reference_id(),
        });
        draft.reset(None);
    };

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
                    <h2 class="card-title">"Spalten"</h2>
                    <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                        {move || Suspend::new(async move {
                            let columns = columns.await.unwrap_or_default();
                            let entities = entities.await.unwrap_or_default();
                            view! { <ColumnTable columns entities actions draft on_add=add_column/> }
                        })}
                    </Transition>
                </div>
            </div>
        </div>
    }
}

/// Bezeichnung der Entität mit Umbenennen.
#[component]
fn EntityHeader(id: i64, name: String, rename: ServerAction<RenameEntity>) -> impl IntoView {
    let editing = RwSignal::new(false);
    let value = RwSignal::new(name.clone());
    let save = move || {
        rename.dispatch(RenameEntity { id, name: value.get_untracked() });
        editing.set(false);
    };

    view! {
        <Show
            when=move || editing.get()
            fallback=move || view! {
                <div class="flex items-center gap-2">
                    <h1 class="text-2xl font-bold">{name.clone()}</h1>
                    <button class="btn btn-ghost btn-sm" title="Umbenennen" on:click=move |_| editing.set(true)>"✎"</button>
                </div>
            }
        >
            <div class="flex items-center gap-2">
                <input
                    class="input"
                    bind:value=value
                    on:keydown=move |ev| if ev.key() == "Enter" { save() }
                />
                <button class="btn btn-primary btn-sm" on:click=move |_| save()>"Speichern"</button>
                <button class="btn btn-ghost btn-sm" on:click=move |_| editing.set(false)>"Abbrechen"</button>
            </div>
        </Show>
    }
}

#[component]
fn ColumnTable(
    columns: Vec<EntityColumn>,
    entities: Vec<Entity>,
    actions: ColumnActions,
    draft: ColumnDraft,
    on_add: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let rows = columns
        .into_iter()
        .enumerate()
        .map(|(index, column)| {
            view! { <ColumnRow column number=index + 1 entities=entities.clone() actions/> }
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
                <tfoot>
                    <tr class="bg-base-200/50">
                        <td>"+"</td>
                        <DraftCells draft entities=entities.clone() on_enter=on_add/>
                        <td class="text-right">
                            <button
                                class="btn btn-primary btn-sm"
                                disabled=move || draft.name.get().trim().is_empty()
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
fn DraftCells(
    draft: ColumnDraft,
    entities: Vec<Entity>,
    on_enter: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let on_keydown = move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Enter" {
            on_enter();
        }
    };
    // Wer einen Verweis wählt, meint in der Regel den Typ "Referenz".
    let on_reference = move |_| {
        if !draft.reference.get_untracked().is_empty() && draft.data_type.get_untracked().is_empty() {
            draft.data_type.set("Referenz".to_string());
        }
    };

    view! {
        <td><input class="input input-sm w-full min-w-32" placeholder="Bezeichnung" bind:value=draft.name on:keydown=on_keydown/></td>
        <td><input class="input input-sm w-full min-w-28" placeholder="Typ" list="column-types" bind:value=draft.data_type on:keydown=on_keydown/></td>
        <td><input class="input input-sm w-full min-w-48" placeholder="Bedeutung" bind:value=draft.meaning on:keydown=on_keydown/></td>
        <td>
            <select class="select select-sm min-w-36" bind:value=draft.reference on:change=on_reference>
                <option value="">"—"</option>
                {entities
                    .into_iter()
                    .map(|e| view! { <option value=e.id.to_string()>{e.name}</option> })
                    .collect_view()}
            </select>
        </td>
    }
}

#[component]
fn ColumnRow(
    column: EntityColumn,
    number: usize,
    entities: Vec<Entity>,
    actions: ColumnActions,
) -> impl IntoView {
    let editing = RwSignal::new(false);
    let draft = ColumnDraft::new(Some(&column));
    let target = column
        .references_entity_id
        .and_then(|id| entities.iter().find(|e| e.id == id).cloned());
    let original = StoredValue::new(column.clone());
    let id = column.id;

    let save = move || {
        if draft.name.get_untracked().trim().is_empty() {
            return;
        }
        actions.update.dispatch(UpdateEntityColumn {
            id,
            name: draft.name.get_untracked(),
            data_type: draft.data_type.get_untracked(),
            meaning: draft.meaning.get_untracked(),
            references_entity_id: draft.reference_id(),
        });
        editing.set(false);
    };
    let cancel = move || {
        original.with_value(|c| draft.reset(Some(c)));
        editing.set(false);
    };

    let display = move || {
        let column = column.clone();
        view! {
            <td class="font-medium">{column.name}</td>
            <td>
                {(!column.data_type.is_empty())
                    .then(|| view! { <span class="badge badge-soft badge-secondary">{column.data_type}</span> })}
            </td>
            <td class="text-base-content/80">{column.meaning}</td>
            <td>
                {target.clone().map(|e| view! {
                    <A href=format!("/entitaeten/{}", e.id) attr:class="link link-hover">{format!("→ {}", e.name)}</A>
                })}
            </td>
            <td class="text-right whitespace-nowrap">
                <button class="btn btn-ghost btn-xs" title="Bearbeiten" on:click=move |_| editing.set(true)>"✎"</button>
                <button class="btn btn-ghost btn-xs text-error" title="Löschen"
                    on:click=move |_| { actions.delete.dispatch(DeleteEntityColumn { id }); }>"✕"</button>
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
            <td class="text-base-content/60">{number}</td>
            <Show when=move || editing.get() fallback=display>
                <DraftCells draft entities=entities.clone() on_enter=save/>
                <td class="text-right whitespace-nowrap">
                    <button class="btn btn-primary btn-xs" on:click=move |_| save()>"Speichern"</button>
                    <button class="btn btn-ghost btn-xs" on:click=move |_| cancel()>"Abbrechen"</button>
                </td>
            </Show>
        </tr>
    }
}
