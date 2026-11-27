use leptos::{html, prelude::*};
use leptos_router::{components::A, hooks::use_params_map};

use super::editor_js;
use crate::backend::bpmn::model::BpmnDiagramContent;
use crate::backend::bpmn::server_fns::{SaveDiagramXml, UpdateDiagram, get_diagram};
use crate::components::modal::FormModal;

/// Seite mit dem grafischen BPMN-2.0-Editor für ein Diagramm.
#[component]
pub fn DiagramEditorPage() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.read().get("id").and_then(|v| v.parse::<i64>().ok());
    let diagram = Resource::new(id, |id| async move {
        match id {
            Some(id) => get_diagram(id).await,
            None => Err(ServerFnError::new("Ungültige Diagrammnummer")),
        }
    });

    view! {
        <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
            {move || Suspend::new(async move {
                match diagram.await {
                    Ok(diagram) => view! { <Editor diagram/> }.into_any(),
                    Err(e) => {
                        view! {
                            <div class="mx-auto max-w-5xl space-y-4">
                                <div class="alert alert-error">{e.to_string()}</div>
                                <A href="/bpmn" attr:class="btn btn-ghost">"← Zur Übersicht"</A>
                            </div>
                        }
                            .into_any()
                    }
                }
            })}
        </Transition>
    }
}

#[component]
fn Editor(diagram: BpmnDiagramContent) -> impl IntoView {
    let id = diagram.id;
    let name = RwSignal::new(diagram.name);
    let description = RwSignal::new(diagram.description);
    // Aktueller Inhalt aus dem Editor und ob er seit dem letzten Speichern geändert wurde.
    let xml = RwSignal::new(diagram.xml);
    let dirty = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    // Token-Simulation aktiv? Dann ist das Modell gesperrt.
    let simulating = RwSignal::new(false);

    let save = ServerAction::<SaveDiagramXml>::new();
    // Der gespeicherte Stand; Änderungen während des Speicherns bleiben ungespeichert markiert.
    let sent = StoredValue::new(String::new());
    let do_save = move || {
        if save.pending().get_untracked() || !dirty.get_untracked() {
            return;
        }
        let content = xml.get_untracked();
        sent.set_value(content.clone());
        error.set(None);
        save.dispatch(SaveDiagramXml { id, xml: content });
    };
    Effect::new(move || match save.value().get() {
        Some(Ok(())) => {
            if sent.with_value(|s| *s == xml.get_untracked()) {
                dirty.set(false);
                editor_js::mark_saved();
            }
        }
        Some(Err(e)) => error.set(Some(e.to_string())),
        None => {}
    });

    // Name und Beschreibung im Modalfenster bearbeiten.
    let update = ServerAction::<UpdateDiagram>::new();
    let modal_open = RwSignal::new(false);
    let draft_name = RwSignal::new(String::new());
    let draft_description = RwSignal::new(String::new());
    let open_modal = move |_| {
        draft_name.set(name.get_untracked());
        draft_description.set(description.get_untracked());
        update.clear();
        modal_open.set(true);
    };
    let submit = move || {
        update.dispatch(UpdateDiagram {
            id,
            name: draft_name.get_untracked(),
            description: draft_description.get_untracked(),
        });
    };
    Effect::new(move || {
        if let Some(Ok(())) = update.value().get() {
            name.set(draft_name.get_untracked().trim().to_string());
            description.set(draft_description.get_untracked().trim().to_string());
            modal_open.set(false);
        }
    });

    // Den Editor erst im Browser erzeugen, sobald die Zeichenfläche im Dokument ist.
    let canvas = NodeRef::<html::Div>::new();
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::prelude::*;
        let initial = xml.get_untracked();
        Effect::new(move || {
            let Some(el) = canvas.get() else { return };
            let on_change = Closure::<dyn FnMut(String)>::new(move |content: String| {
                xml.set(content);
                dirty.set(true);
            })
            .into_js_value();
            let on_save = Closure::<dyn FnMut()>::new(do_save).into_js_value();
            let on_error = Closure::<dyn FnMut(String)>::new(move |message: String| error.set(Some(message)))
                .into_js_value();
            let on_simulation = Closure::<dyn FnMut(bool)>::new(move |active: bool| simulating.set(active))
                .into_js_value();
            editor_js::mount(el.as_ref(), &initial, &on_change, &on_save, &on_error, &on_simulation);
        });
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = (canvas, do_save);
    on_cleanup(editor_js::destroy);

    let file_name = move || name.get_untracked();

    view! {
        <div class="flex h-[calc(100vh-8rem)] min-h-[32rem] flex-col gap-4 lg:h-[calc(100vh-10rem)]">
            <div class="flex flex-wrap items-start justify-between gap-4">
                <div class="min-w-0">
                    <div class="breadcrumbs py-0 text-sm">
                        <ul>
                            <li><A href="/bpmn">"BPMN-Modellierung"</A></li>
                            <li>{move || name.get()}</li>
                        </ul>
                    </div>
                    <h1
                        class="cursor-pointer truncate text-2xl font-bold"
                        title="Doppelklick zum Bearbeiten"
                        on:dblclick=move |_| open_modal(())
                    >
                        {move || name.get()}
                    </h1>
                    <p class="text-base-content/70">{move || description.get()}</p>
                </div>
                <div class="flex flex-wrap items-center gap-2">
                    {move || {
                        if save.pending().get() {
                            view! { <span class="badge badge-ghost">"Wird gespeichert …"</span> }.into_any()
                        } else if dirty.get() {
                            view! { <span class="badge badge-warning">"Ungespeicherte Änderungen"</span> }.into_any()
                        } else {
                            view! { <span class="badge badge-success badge-soft">"Gespeichert"</span> }.into_any()
                        }
                    }}
                    <button class="btn btn-ghost btn-sm" on:click=move |_| open_modal(())>"Bearbeiten"</button>
                    <button
                        class="btn btn-primary btn-sm"
                        title="Speichern (Strg+S)"
                        disabled=move || !dirty.get() || save.pending().get()
                        on:click=move |_| do_save()
                    >
                        "Speichern"
                    </button>
                </div>
            </div>

            {move || {
                error.get().map(|message| {
                    view! {
                        <div class="alert alert-error">
                            <span>{message}</span>
                            <button class="btn btn-ghost btn-sm" on:click=move |_| error.set(None)>"Schliessen"</button>
                        </div>
                    }
                })
            }}

            <div class="flex flex-wrap items-center gap-2">
                <button
                    class="btn btn-sm"
                    class:btn-accent=move || !simulating.get()
                    class:btn-error=move || simulating.get()
                    title="Token-Simulation ein- oder ausschalten (T)"
                    on:click=move |_| editor_js::toggle_simulation()
                >
                    {move || if simulating.get() { "■ Simulation beenden" } else { "▶ Simulation" }}
                </button>
                <div class="join">
                    <button class="btn btn-sm join-item" title="Rückgängig (Strg+Z)" disabled=move || simulating.get() on:click=move |_| editor_js::undo()>
                        "↶ Rückgängig"
                    </button>
                    <button class="btn btn-sm join-item" title="Wiederholen (Strg+Y)" disabled=move || simulating.get() on:click=move |_| editor_js::redo()>
                        "↷ Wiederholen"
                    </button>
                </div>
                <div class="join">
                    <button class="btn btn-sm join-item" title="Verkleinern" on:click=move |_| editor_js::zoom(-1)>"−"</button>
                    <button class="btn btn-sm join-item" title="Ganzes Diagramm anzeigen" on:click=move |_| editor_js::zoom(0)>
                        "Einpassen"
                    </button>
                    <button class="btn btn-sm join-item" title="Vergrössern" on:click=move |_| editor_js::zoom(1)>"+"</button>
                </div>
                <div class="join">
                    <button class="btn btn-sm join-item" title="BPMN-2.0-Datei (.bpmn) laden" disabled=move || simulating.get() on:click=move |_| editor_js::import_file()>
                        "Importieren"
                    </button>
                    <button class="btn btn-sm join-item" title="Als BPMN-2.0-XML herunterladen" on:click=move |_| editor_js::export_xml(&file_name())>
                        "Export .bpmn"
                    </button>
                    <button class="btn btn-sm join-item" title="Als Grafik herunterladen" on:click=move |_| editor_js::export_svg(&file_name())>
                        "Export .svg"
                    </button>
                </div>
                <span class="text-sm text-base-content/60">
                    {move || {
                        if simulating.get() {
                            "Simulation: ▶ am Startereignis schickt einen Token los. An exklusiven Gateways den Fluss wählen, \
                             an Zwischenereignissen auslösen. Das Modell ist währenddessen gesperrt."
                        } else {
                            "Elemente aus der Palette links ziehen, Doppelklick zum Beschriften, Schraubenschlüssel ändert den Typ."
                        }
                    }}
                </span>
            </div>

            <div
                class="relative flex-1 overflow-hidden rounded-box border bg-base-100 shadow-sm"
                class:border-base-300=move || !simulating.get()
                class:border-accent=move || simulating.get()
            >
                <div node_ref=canvas class="bpmn-canvas absolute inset-0"></div>
            </div>

            <FormModal
                open=modal_open
                title="Diagramm bearbeiten".to_string()
                on_submit=submit
                submit_disabled=Signal::derive(move || update.pending().get() || draft_name.get().trim().is_empty())
            >
                <label class="floating-label">
                    <span>"Name"</span>
                    <input class="input w-full" placeholder="Name" required bind:value=draft_name/>
                </label>
                <label class="floating-label">
                    <span>"Beschreibung"</span>
                    <textarea class="textarea w-full" placeholder="Beschreibung" bind:value=draft_description></textarea>
                </label>
                {move || {
                    update.value().get().and_then(Result::err).map(|e| {
                        view! { <div class="alert alert-error">{e.to_string()}</div> }
                    })
                }}
            </FormModal>
        </div>
    }
}
