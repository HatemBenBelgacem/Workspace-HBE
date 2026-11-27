use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    ParamSegment, StaticSegment,
    components::{Route, Router, Routes},
};

use crate::components::bpmn::components::{
    diagram_editor::DiagramEditorPage, diagram_list::DiagramsPage,
};
use crate::components::entitaet::components::{
    entity_detail::EntityDetailPage, entity_list::EntitiesPage,
};
use crate::components::prozess::components::{
    process_detail::ProcessDetailPage, process_list::ProcessesPage,
};
use crate::components::{home::HomePage, layout::AppLayout};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="de" data-theme="light">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
                // Brücke zum BPMN-Editor; lädt die Bibliothek erst, wenn ein Diagramm geöffnet wird.
                <script src="/bpmn-editor.js"></script>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/workspace-hbe.css"/>
        <Title text="Workspace HBE"/>

        <Router>
            <AppLayout>
                <Routes fallback=|| view! { <p>"Seite nicht gefunden."</p> }>
                    <Route path=StaticSegment("") view=HomePage/>
                    <Route path=StaticSegment("prozesse") view=ProcessesPage/>
                    <Route path=(StaticSegment("prozesse"), ParamSegment("id")) view=ProcessDetailPage/>
                    <Route path=StaticSegment("bpmn") view=DiagramsPage/>
                    <Route path=(StaticSegment("bpmn"), ParamSegment("id")) view=DiagramEditorPage/>
                    <Route path=StaticSegment("entitaeten") view=EntitiesPage/>
                    <Route path=(StaticSegment("entitaeten"), ParamSegment("id")) view=EntityDetailPage/>
                </Routes>
            </AppLayout>
        </Router>
    }
}
