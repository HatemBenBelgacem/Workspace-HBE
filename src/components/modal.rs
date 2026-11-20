//! Gemeinsames Modalfenster für alle Eingabemasken.

use std::time::Duration;

use leptos::{html, prelude::*};

/// Modalfenster mit Formular.
///
/// Enter oder der Hauptknopf rufen `on_submit` auf. "Abbrechen", Escape und ein Klick neben das
/// Fenster schliessen es ohne zu speichern. Beim Öffnen erhält das erste Feld den Fokus.
#[component]
pub fn FormModal(
    open: RwSignal<bool>,
    #[prop(into)] title: Signal<String>,
    on_submit: impl Fn() + Copy + Send + Sync + 'static,
    #[prop(default = "Speichern")] submit_label: &'static str,
    #[prop(into, default = Signal::stored(false))] submit_disabled: Signal<bool>,
    children: Children,
) -> impl IntoView {
    let form_ref = NodeRef::<html::Form>::new();

    Effect::new(move || {
        if !open.get() {
            return;
        }
        // Kurz warten, bis das Fenster sichtbar ist; unsichtbare Felder nehmen keinen Fokus an.
        set_timeout(
            move || {
                use wasm_bindgen::JsCast;
                let field = form_ref
                    .get_untracked()
                    .and_then(|form| form.query_selector("input, select, textarea").ok().flatten())
                    .and_then(|el| el.dyn_into::<leptos::web_sys::HtmlElement>().ok());
                if let Some(field) = field {
                    let _ = field.focus();
                }
            },
            Duration::from_millis(50),
        );
    });

    let close = move || open.set(false);

    view! {
        <div
            class="modal"
            class:modal-open=move || open.get()
            role="dialog"
            aria-modal="true"
            aria-hidden=move || (!open.get()).to_string()
            on:keydown=move |ev| {
                if ev.key() == "Escape" {
                    close();
                }
            }
        >
            <form
                node_ref=form_ref
                class="modal-box space-y-4"
                on:submit=move |ev| {
                    ev.prevent_default();
                    if !submit_disabled.get_untracked() {
                        on_submit();
                    }
                }
            >
                <h3 class="text-lg font-bold">{move || title.get()}</h3>
                {children()}
                <div class="modal-action">
                    <button type="button" class="btn btn-ghost" on:click=move |_| close()>"Abbrechen"</button>
                    <button type="submit" class="btn btn-primary" disabled=move || submit_disabled.get()>
                        {submit_label}
                    </button>
                </div>
            </form>
            <div class="modal-backdrop" on:click=move |_| close()></div>
        </div>
    }
}
