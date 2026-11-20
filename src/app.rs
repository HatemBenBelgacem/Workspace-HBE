use leptos::prelude::*;
use leptos_meta::{HashedStylesheet, MetaTags, Title, provide_meta_context};
use leptos_router::{
    ParamSegment, StaticSegment,
    components::{Route, Router, Routes},
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
                // Dateinamen mit Hash (hash-files in Cargo.toml), damit Browser nach einem
                // Deployment nie altes CSS, JavaScript oder WebAssembly aus dem Cache verwenden.
                <HashedStylesheet options=options.clone() id="leptos"/>
                <HydrationScripts options/>
                <MetaTags/>
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
        <Title text="Workspace HBE"/>

        <Router>
            <AppLayout>
                <Routes fallback=|| view! { <p>"Seite nicht gefunden."</p> }>
                    <Route path=StaticSegment("") view=HomePage/>
                    <Route path=StaticSegment("prozesse") view=ProcessesPage/>
                    <Route path=(StaticSegment("prozesse"), ParamSegment("id")) view=ProcessDetailPage/>
                    <Route path=StaticSegment("entitaeten") view=EntitiesPage/>
                    <Route path=(StaticSegment("entitaeten"), ParamSegment("id")) view=EntityDetailPage/>
                </Routes>
            </AppLayout>
        </Router>
    }
}
