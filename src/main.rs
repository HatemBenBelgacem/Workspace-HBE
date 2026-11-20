#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use workspace_hbe::app::*;

    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL ist nicht gesetzt");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Verbindung zur Datenbank fehlgeschlagen");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Migrationen fehlgeschlagen");

    let conf = get_configuration(None).unwrap();
    // Hosting-Plattformen wie Railway geben den Port über PORT vor und erwarten, dass auf
    // allen Schnittstellen gelauscht wird.
    let addr = match std::env::var("PORT").ok().and_then(|p| p.parse::<u16>().ok()) {
        Some(port) => std::net::SocketAddr::from(([0, 0, 0, 0], port)),
        None => conf.leptos_options.site_addr,
    };
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            move || provide_context(pool.clone()),
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // Kein Client-seitiger Einstiegspunkt; siehe lib.rs für die Hydration.
}
