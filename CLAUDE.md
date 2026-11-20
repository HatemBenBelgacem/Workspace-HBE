# CLAUDE.md

Workspace HBE ist eine Web-App für **Prozess- und Datenmodellierung**: Prozesse mit Schritten,
Bedingungen und Verzweigungen sowie Entitäten mit Spalten und Fremdschlüssel-Verweisen (ER-Diagramm).

Die Sprache im Projekt ist **Deutsch**: Oberfläche, Fehlermeldungen, Kommentare und Commit-Nachrichten.

## Technik

- **Rust** (Edition 2024), **Leptos 0.8** als Fullstack-Framework (SSR + Hydration)
- **Axum** als Webserver, **SQLx** mit **PostgreSQL**
- **Tailwind CSS 4** mit **DaisyUI 5** (`styles/input.css`)
- Build mit **cargo-leptos**
- Deployment: Docker-Image (`Dockerfile`) auf **Railway** (`railway.toml`)

## Befehle

Datenbank lokal starten (Postgres auf Port 5433):

```bash
docker compose up -d db
```

`.env` im Projektordner (nicht eingecheckt):

```
DATABASE_URL=postgres://hbe:hbe@localhost:5433/workspace_hbe
```

Einmalig Node-Pakete für Tailwind installieren:

```bash
npm install
```

Entwicklungsserver mit Live-Reload (http://127.0.0.1:3010):

```bash
cargo leptos watch
```

Release-Build:

```bash
cargo leptos build --release
```

Schnelle Prüfung beider Ziele:

```bash
cargo check --features ssr
cargo check --features hydrate --target wasm32-unknown-unknown
```

Migrationen laufen beim Serverstart automatisch (`sqlx::migrate!()` in `src/main.rs`).

## Architektur

Der Code ist **pro Fachbereich** aufgeteilt. Aktuell gibt es die Bereiche `entität` und `prozess`.
Jeder Bereich hat diese Struktur:

- `src/backend/<bereich>/model.rs` – Datenstrukturen (Serde, mit `sqlx::FromRow` nur bei `ssr`)
- `src/backend/<bereich>/server_fns.rs` – Leptos-`#[server]`-Funktionen mit den SQL-Abfragen
- `src/components/<bereich>/components/` – Leptos-Komponenten (Seiten, Listen, Detailansichten)

Weitere Dateien:

- `src/main.rs` – Server-Start: DB-Pool, Migrationen, Axum-Router
- `src/lib.rs` – Einstiegspunkt für die Hydration im Browser
- `src/app.rs` – HTML-Gerüst (`shell`) und Routen
- `src/backend/db.rs` – gemeinsame Hilfen für Server-Funktionen (`pool()`, `err()`, `required()`)
- `src/components/layout.rs`, `home.rs` – Rahmen und Startseite
- `src/components/modal.rs` – gemeinsames Modalfenster für alle Eingabemasken
- `migrations/` – SQL-Migrationen, fortlaufend nummeriert (`0006_….sql`)

## Konventionen

- **Ordner `entität` mit Umlaut:** Rust erlaubt keine Umlaute in Modulnamen. Das Modul heisst
  `entitaet` und wird mit `#[path = "entität/mod.rs"]` eingebunden. Neue Bereiche mit Umlaut genauso lösen.
- **Server-only-Code** mit `#[cfg(feature = "ssr")]` absichern (z. B. `use crate::backend::db`),
  sonst bricht der WebAssembly-Build.
- SQL mit `sqlx::query` / `query_as` / `query_scalar` zur Laufzeit, **keine** `query!`-Makros
  (es gibt keinen Offline-Cache für SQLx).
- Fehler in Server-Funktionen über `db::err` umwandeln, Pflichtfelder mit `db::required` prüfen.
- Bestehende Migrationen nie ändern, sondern eine neue Datei anlegen.
- Styling mit Tailwind- und DaisyUI-Klassen direkt in den `view!`-Makros.
- Routen auf Deutsch (`/prozesse`, `/entitaeten`).
- **Jeder Datensatz öffnet sich per Doppelklick** (`on:dblclick` auf der Zeile): In Übersichtslisten
  führt er zur Detailseite, in Detailtabellen (Spalten, Schritte) öffnet er das Modalfenster zum Bearbeiten.
  Neue Tabellen müssen das ebenfalls unterstützen.
- **Eingabemasken immer im Modalfenster**, nie inline in Tabellen oder Seiten. Dafür gibt es die
  gemeinsame Komponente `FormModal` in `src/components/modal.rs` (Enter speichert, Escape schliesst).
  Pro Seite ein Modalfenster für Neu und Bearbeiten, gesteuert über Signale.
