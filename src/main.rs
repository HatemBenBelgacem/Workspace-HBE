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
        .layer(axum::middleware::from_fn(cache_control))
        .with_state(leptos_options);

    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

/// Legt fest, wie lange der Browser Antworten zwischenspeichern darf.
///
/// Dateien unter /pkg tragen einen Hash im Namen (`hash-files` in Cargo.toml) und ändern sich nie,
/// sie dürfen dauerhaft im Cache bleiben. Alles andere (Seiten, Server-Funktionen, Dateien aus
/// `public/`) muss der Browser bei jedem Aufruf prüfen, sonst passt nach einem Deployment
/// die zwischengespeicherte Oberfläche nicht mehr zum Server.
#[cfg(feature = "ssr")]
async fn cache_control(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::http::{HeaderValue, header::CACHE_CONTROL};

    let immutable = request.uri().path().starts_with("/pkg/");
    let mut response = next.run(request).await;
    let value = if immutable && response.status().is_success() {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    response.headers_mut().insert(CACHE_CONTROL, HeaderValue::from_static(value));
    response
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // Kein Client-seitiger Einstiegspunkt; siehe lib.rs für die Hydration.
}
