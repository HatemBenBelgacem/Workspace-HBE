use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_navigate};

use crate::backend::prozess::model::Process;
use crate::backend::prozess::server_fns::{CreateProcess, DeleteProcess, list_processes};

#[component]
pub fn ProcessesPage() -> impl IntoView {
    let create = ServerAction::<CreateProcess>::new();
    let delete = ServerAction::<DeleteProcess>::new();
    let processes = Resource::new(move || delete.version().get(), |_| list_processes());

    let name = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());

    // Nach dem Anlegen direkt zur Erfassung der Schritte wechseln.
    let navigate = use_navigate();
    Effect::new(move || {
        if let Some(Ok(id)) = create.value().get() {
            navigate(&format!("/prozesse/{id}"), Default::default());
        }
    });

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        create.dispatch(CreateProcess {
            name: name.get_untracked(),
            description: description.get_untracked(),
        });
    };

    view! {
        <div class="mx-auto max-w-5xl space-y-6">
            <h1 class="text-2xl font-bold">"Geschäftsprozesse"</h1>

            <form class="card bg-base-100 shadow-sm" on:submit=submit>
                <div class="card-body gap-4">
                    <h2 class="card-title">"Neuer Prozess"</h2>
                    <div class="grid gap-4 md:grid-cols-2">
                        <label class="floating-label">
                            <span>"Name"</span>
                            <input class="input w-full" placeholder="Name" required bind:value=name/>
                        </label>
                        <label class="floating-label">
                            <span>"Beschreibung"</span>
                            <input class="input w-full" placeholder="Beschreibung" bind:value=description/>
                        </label>
                    </div>
                    {move || {
                        create.value().get().and_then(Result::err).map(|e| {
                            view! { <div class="alert alert-error">{e.to_string()}</div> }
                        })
                    }}
                    <div class="card-actions justify-end">
                        <button class="btn btn-primary" type="submit" disabled=move || create.pending().get()>
                            "Anlegen und Schritte erfassen"
                        </button>
                    </div>
                </div>
            </form>

            <div class="card bg-base-100 shadow-sm">
                <div class="card-body">
                    <h2 class="card-title">"Erfasste Prozesse"</h2>
                    <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                        {move || Suspend::new(async move {
                            match processes.await {
                                Ok(list) if list.is_empty() => {
                                    view! { <p class="text-base-content/60">"Noch keine Prozesse erfasst."</p> }
                                        .into_any()
                                }
                                Ok(list) => view! { <ProcessTable list delete/> }.into_any(),
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
fn ProcessTable(list: Vec<Process>, delete: ServerAction<DeleteProcess>) -> impl IntoView {
    let navigate = use_navigate();
    view! {
        <div class="overflow-x-auto">
            <table class="table">
                <thead>
                    <tr>
                        <th>"Name"</th>
                        <th>"Beschreibung"</th>
                        <th class="w-48"></th>
                    </tr>
                </thead>
                <tbody>
                    {list
                        .into_iter()
                        .map(|p| {
                            // Zweistufiges Löschen, weil dabei alle Schritte des Prozesses verloren gehen.
                            let confirming = RwSignal::new(false);
                            let id = p.id;
                            let open = {
                                let navigate = navigate.clone();
                                move |_| navigate(&format!("/prozesse/{id}"), Default::default())
                            };
                            view! {
                                <tr class="hover:bg-base-200 cursor-pointer select-none" title="Doppelklick zum Öffnen" on:dblclick=open>
                                    <td class="font-medium">
                                        <A href=format!("/prozesse/{id}") attr:class="link link-hover">{p.name}</A>
                                    </td>
                                    <td class="text-base-content/70">{p.description}</td>
                                    <td class="text-right">
                                        <A href=format!("/prozesse/{id}") attr:class="btn btn-ghost btn-sm">"Öffnen"</A>
                                        <button
                                            class="btn btn-sm"
                                            class:btn-ghost=move || !confirming.get()
                                            class:btn-error=move || confirming.get()
                                            on:click=move |_| {
                                                if confirming.get() {
                                                    delete.dispatch(DeleteProcess { id });
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
