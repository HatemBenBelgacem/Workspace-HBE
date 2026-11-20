use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_location};

struct NavItem {
    href: &'static str,
    label: &'static str,
    icon: &'static str,
}

const NAV_ITEMS: &[NavItem] = &[
    NavItem {
        href: "/",
        label: "Übersicht",
        icon: "M2.25 12l8.954-8.955a1.126 1.126 0 011.591 0L21.75 12M4.5 9.75v10.125c0 .621.504 1.125 1.125 1.125H9.75v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21h4.125c.621 0 1.125-.504 1.125-1.125V9.75M8.25 21h8.25",
    },
    NavItem {
        href: "/prozesse",
        label: "Geschäftsprozesse",
        icon: "M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5",
    },
    NavItem {
        href: "/entitaeten",
        label: "Entitäten",
        icon: "M20.25 6.375c0 2.278-3.694 4.125-8.25 4.125S3.75 8.653 3.75 6.375m16.5 0c0-2.278-3.694-4.125-8.25-4.125S3.75 4.097 3.75 6.375m16.5 0v11.25c0 2.278-3.694 4.125-8.25 4.125s-8.25-1.847-8.25-4.125V6.375m16.5 0v3.75m-16.5-3.75v3.75m16.5 0v3.75C20.25 16.153 16.556 18 12 18s-8.25-1.847-8.25-4.125v-3.75m16.5 0c0 2.278-3.694 4.125-8.25 4.125s-8.25-1.847-8.25-4.125",
    },
];

/// Grundgerüst jeder Seite: Header oben, Navigation in der Sidebar links.
/// Auf kleinen Bildschirmen wird die Sidebar über den Button im Header eingeblendet.
#[component]
pub fn AppLayout(children: Children) -> impl IntoView {
    let sidebar_open = RwSignal::new(false);

    view! {
        <div class="drawer lg:drawer-open">
            <input id="app-sidebar" type="checkbox" class="drawer-toggle" bind:checked=sidebar_open/>
            <div class="drawer-content flex min-h-screen flex-col">
                <Header/>
                <main class="flex-1 bg-base-200 p-4 lg:p-8">{children()}</main>
            </div>
            <div class="drawer-side z-40">
                <label for="app-sidebar" aria-label="Navigation schliessen" class="drawer-overlay"></label>
                <Sidebar on_navigate=move || sidebar_open.set(false)/>
            </div>
        </div>
    }
}

#[component]
fn Header() -> impl IntoView {
    view! {
        <header class="navbar sticky top-0 z-30 border-b border-base-300 bg-base-100 px-4">
            <div class="flex-none lg:hidden">
                <label for="app-sidebar" aria-label="Navigation öffnen" class="btn btn-square btn-ghost">
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" class="size-6">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5"/>
                    </svg>
                </label>
            </div>
            <div class="flex-1">
                <span class="text-lg font-semibold">"Workspace HBE"</span>
            </div>
            <div class="flex-none">
                <span class="badge badge-ghost">"Prozessmanagement"</span>
            </div>
        </header>
    }
}

#[component]
fn Sidebar(on_navigate: impl Fn() + Copy + Send + Sync + 'static) -> impl IntoView {
    let location = use_location();
    let is_active = move |href: &str| {
        let path = location.pathname.get();
        if href == "/" { path == "/" } else { path.starts_with(href) }
    };

    view! {
        <aside class="flex min-h-full w-64 flex-col border-r border-base-300 bg-base-100">
            <div class="flex h-16 items-center gap-2 border-b border-base-300 px-6">
                <div class="grid size-8 place-items-center rounded-lg bg-primary font-bold text-primary-content">"H"</div>
                <span class="font-semibold">"Workspace HBE"</span>
            </div>
            <ul class="menu w-full gap-1 p-4">
                <li class="menu-title">"Navigation"</li>
                {NAV_ITEMS
                    .iter()
                    .map(|item| {
                        let href = item.href;
                        view! {
                            <li>
                                <A href=href on:click=move |_| on_navigate()
                                    attr:class=move || if is_active(href) { "menu-active" } else { "" }>
                                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-5">
                                        <path stroke-linecap="round" stroke-linejoin="round" d=item.icon/>
                                    </svg>
                                    {item.label}
                                </A>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
        </aside>
    }
}
