use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_navigate};

use crate::backend::bpmn::model::BpmnDiagram;
use crate::backend::bpmn::server_fns::{CreateDiagram, DeleteDiagram, list_diagrams};
use crate::components::modal::FormModal;

/// Übersicht der frei modellierten BPMN-2.0-Diagramme.
#[component]
pub fn DiagramsPage() -> impl IntoView {
    let create = ServerAction::<CreateDiagram>::new();
    let delete = ServerAction::<DeleteDiagram>::new();
    let diagrams = Resource::new(move || delete.version().get(), |_| list_diagrams());

    let name = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let modal_open = RwSignal::new(false);

    // Nach dem Anlegen direkt in den Editor wechseln.
    let navigate = use_navigate();
    Effect::new(move || {
        if let Some(Ok(id)) = create.value().get() {
            navigate(&format!("/bpmn/{id}"), Default::default());
        }
    });

    let open_modal = move |_| {
        name.set(String::new());
        description.set(String::new());
        create.clear();
        modal_open.set(true);
    };
    let submit = move || {
        create.dispatch(CreateDiagram {
            name: name.get_untracked(),
            description: description.get_untracked(),
        });
    };

    view! {
        <div class="mx-auto max-w-5xl space-y-6">
            <div class="flex items-center justify-between gap-4">
                <div>
                    <h1 class="text-2xl font-bold">"BPMN-Modellierung"</h1>
                    <p class="text-base-content/70">
                        "Prozesse frei als BPMN-2.0-Diagramm zeichnen, als .bpmn importieren und exportieren."
                    </p>
                </div>
                <button class="btn btn-primary" on:click=open_modal>"+ Neues Diagramm"</button>
            </div>

            <FormModal
                open=modal_open
                title="Neues BPMN-Diagramm".to_string()
                on_submit=submit
                submit_label="Anlegen und modellieren"
                submit_disabled=Signal::derive(move || create.pending().get() || name.get().trim().is_empty())
            >
                <label class="floating-label">
                    <span>"Name"</span>
                    <input class="input w-full" placeholder="Name" required bind:value=name/>
                </label>
                <label class="floating-label">
                    <span>"Beschreibung"</span>
                    <textarea class="textarea w-full" placeholder="Beschreibung" bind:value=description></textarea>
                </label>
                {move || {
                    create.value().get().and_then(Result::err).map(|e| {
                        view! { <div class="alert alert-error">{e.to_string()}</div> }
                    })
                }}
            </FormModal>

            <div class="card bg-base-100 shadow-sm">
                <div class="card-body">
                    <h2 class="card-title">"Diagramme"</h2>
                    <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                        {move || Suspend::new(async move {
                            match diagrams.await {
                                Ok(list) if list.is_empty() => {
                                    view! { <p class="text-base-content/60">"Noch keine Diagramme angelegt."</p> }
                                        .into_any()
                                }
                                Ok(list) => view! { <DiagramTable list delete/> }.into_any(),
                                Err(e) => view! { <div class="alert alert-error">{e.to_string()}</div> }.into_any(),
                            }
                        })}
                    </Transition>
                </div>
            </div>
        </div>
    }
}

#[component]
fn DiagramTable(list: Vec<BpmnDiagram>, delete: ServerAction<DeleteDiagram>) -> impl IntoView {
    let navigate = use_navigate();
    view! {
        <div class="overflow-x-auto">
            <table class="table">
                <thead>
                    <tr>
                        <th>"Name"</th>
                        <th>"Beschreibung"</th>
                        <th>"Geändert"</th>
                        <th class="w-48"></th>
                    </tr>
                </thead>
                <tbody>
                    {list
                        .into_iter()
                        .map(|d| {
                            // Zweistufiges Löschen, weil dabei das ganze Diagramm verloren geht.
                            let confirming = RwSignal::new(false);
                            let id = d.id;
                            let open = {
                                let navigate = navigate.clone();
                                move |_| navigate(&format!("/bpmn/{id}"), Default::default())
                            };
                            view! {
                                <tr class="hover:bg-base-200 cursor-pointer select-none" title="Doppelklick zum Öffnen" on:dblclick=open>
                                    <td class="font-medium">
                                        <A href=format!("/bpmn/{id}") attr:class="link link-hover">{d.name}</A>
                                    </td>
                                    <td class="text-base-content/70">{d.description}</td>
                                    <td class="whitespace-nowrap text-base-content/70">{d.updated}</td>
                                    <td class="text-right whitespace-nowrap">
                                        <A href=format!("/bpmn/{id}") attr:class="btn btn-ghost btn-sm">"Öffnen"</A>
                                        <button
                                            class="btn btn-sm"
                                            class:btn-ghost=move || !confirming.get()
                                            class:btn-error=move || confirming.get()
                                            on:click=move |_| {
                                                if confirming.get() {
                                                    delete.dispatch(DeleteDiagram { id });
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
