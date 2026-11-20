use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_navigate};

use super::er_diagram::ErDiagram;
use crate::backend::entitaet::model::Entity;
use crate::backend::entitaet::server_fns::{
    CreateEntity, DeleteEntity, list_all_entity_columns, list_entities,
};

#[derive(Clone, Copy, PartialEq)]
enum EntitiesTab {
    List,
    Diagram,
}

impl EntitiesTab {
    fn label(self) -> &'static str {
        match self {
            Self::List => "Liste",
            Self::Diagram => "ER-Diagramm",
        }
    }
}

#[component]
pub fn EntitiesPage() -> impl IntoView {
    let create = ServerAction::<CreateEntity>::new();
    let delete = ServerAction::<DeleteEntity>::new();
    let entities = Resource::new(move || delete.version().get(), |_| list_entities());
    let columns = Resource::new(move || delete.version().get(), |_| list_all_entity_columns());
    let active_tab = RwSignal::new(EntitiesTab::List);

    let name = RwSignal::new(String::new());

    // Nach dem Anlegen direkt zur Erfassung der Spalten wechseln.
    let navigate = use_navigate();
    Effect::new(move || {
        if let Some(Ok(id)) = create.value().get() {
            navigate(&format!("/entitaeten/{id}"), Default::default());
        }
    });

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        create.dispatch(CreateEntity { name: name.get_untracked() });
    };

    view! {
        <div class="mx-auto max-w-6xl space-y-6">
            <h1 class="text-2xl font-bold">"Entitäten"</h1>

            <form class="card bg-base-100 shadow-sm" on:submit=submit>
                <div class="card-body gap-4">
                    <h2 class="card-title">"Neue Entität"</h2>
                    <label class="floating-label">
                        <span>"Bezeichnung"</span>
                        <input class="input w-full" placeholder="Bezeichnung" required bind:value=name/>
                    </label>
                    {move || {
                        create.value().get().and_then(Result::err).map(|e| {
                            view! { <div class="alert alert-error">{e.to_string()}</div> }
                        })
                    }}
                    <div class="card-actions justify-end">
                        <button class="btn btn-primary" type="submit" disabled=move || create.pending().get()>
                            "Anlegen und Spalten erfassen"
                        </button>
                    </div>
                </div>
            </form>

            <div class="card bg-base-100 shadow-sm">
                <div role="tablist" class="tabs tabs-border px-4 pt-2">
                    {[EntitiesTab::List, EntitiesTab::Diagram]
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
                            let list = match entities.await {
                                Ok(list) => list,
                                Err(e) => {
                                    return view! { <div class="alert alert-error">{e.to_string()}</div> }
                                        .into_any();
                                }
                            };
                            match active_tab.get() {
                                EntitiesTab::List if list.is_empty() => {
                                    view! { <p class="text-base-content/60">"Noch keine Entitäten erfasst."</p> }
                                        .into_any()
                                }
                                EntitiesTab::List => view! { <EntityTable list delete/> }.into_any(),
                                EntitiesTab::Diagram => {
                                    let columns = columns.await.unwrap_or_default();
                                    view! { <ErDiagram entities=list columns/> }.into_any()
                                }
                            }
                        })}
                    </Transition>
                </div>
            </div>
        </div>
    }
}

#[component]
fn EntityTable(list: Vec<Entity>, delete: ServerAction<DeleteEntity>) -> impl IntoView {
    let navigate = use_navigate();
    view! {
        <div class="overflow-x-auto">
            <table class="table">
                <thead>
                    <tr>
                        <th>"Bezeichnung"</th>
                        <th class="w-24">"Spalten"</th>
                        <th class="w-48"></th>
                    </tr>
                </thead>
                <tbody>
                    {list
                        .into_iter()
                        .map(|entity| {
                            // Zweistufiges Löschen, weil dabei alle Spalten der Entität verloren gehen.
                            let confirming = RwSignal::new(false);
                            let id = entity.id;
                            let open = {
                                let navigate = navigate.clone();
                                move |_| navigate(&format!("/entitaeten/{id}"), Default::default())
                            };
                            view! {
                                <tr class="hover:bg-base-200 cursor-pointer select-none" title="Doppelklick zum Öffnen" on:dblclick=open>
                                    <td class="font-medium">
                                        <A href=format!("/entitaeten/{id}") attr:class="link link-hover">{entity.name}</A>
                                    </td>
                                    <td><span class="badge badge-ghost">{entity.column_count}</span></td>
                                    <td class="text-right">
                                        <A href=format!("/entitaeten/{id}") attr:class="btn btn-ghost btn-sm">"Öffnen"</A>
                                        <button
                                            class="btn btn-sm"
                                            class:btn-ghost=move || !confirming.get()
                                            class:btn-error=move || confirming.get()
                                            on:click=move |_| {
                                                if confirming.get() {
                                                    delete.dispatch(DeleteEntity { id });
                                                } else {
                                                    confirming.set(true);
                                                }
                                            }
                                        >
                                            {move || if confirming.get() { "Wirklich löschen?" } else { "Löschen" }}
                                        </button>
                                    </td>
                                </tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </table>
        </div>
    }
}
