use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-4xl space-y-6">
            <h1 class="text-2xl font-bold">"Übersicht"</h1>
            <div class="card bg-base-100 shadow-sm">
                <div class="card-body">
                    <h2 class="card-title">"Geschäftsprozesse erfassen"</h2>
                    <p>
                        "Beschreibe die Schritte eines Prozesses in einer Tabelle mit Aktivität, Rolle, Input, Output und System. "
                        "Daneben entsteht automatisch das Prozessmodell mit einer Bahn pro Rolle."
                    </p>
                    <div class="card-actions justify-end">
                        <A href="/prozesse" attr:class="btn btn-primary">"Zu den Prozessen"</A>
                    </div>
                </div>
            </div>
            <div class="card bg-base-100 shadow-sm">
                <div class="card-body">
                    <h2 class="card-title">"BPMN-Modellierung"</h2>
                    <p>
                        "Zeichne Prozesse frei als BPMN-2.0-Diagramm mit Pools, Bahnen, Ereignissen, Gateways und Aufgaben. "
                        "Diagramme lassen sich als .bpmn-Datei importieren und exportieren."
                    </p>
                    <div class="card-actions justify-end">
                        <A href="/bpmn" attr:class="btn btn-primary">"Zum BPMN-Editor"</A>
                    </div>
                </div>
            </div>
        </div>
    }
}
