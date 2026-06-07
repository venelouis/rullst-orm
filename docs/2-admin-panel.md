# Rullst Admin Panel

Rullst ORM includes a completely free, drop-in web dashboard that you can serve from any Rust web framework. It renders a beautiful dark-mode interface for managing your database tables.

Unlike traditional ORMs that charge for their studio/admin tools, Rullst provides this natively — zero dependencies, zero compilation, just a static HTML string.

## How it Works

The Admin Panel is a self-contained HTML/CSS/JS string returned by `rullst_orm::admin::dashboard_html()`. You mount it on any route in your web framework.

### Example using Axum

```rust
use axum::{routing::get, Router, response::Html};
use rullst_orm::admin::dashboard_html;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/admin", get(|| async { Html(dashboard_html()) }));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Admin Panel available at http://localhost:3000/admin");

    axum::serve(listener, app).await.unwrap();
}
```

### Example using Actix-web

```rust
use actix_web::{web, App, HttpServer, HttpResponse};
use rullst_orm::admin::dashboard_html;

async fn admin_handler() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(dashboard_html())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().route("/admin", web::get().to(admin_handler))
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}
```

## Audit Logs

The Admin Panel has a built-in Audit Logs section. To populate it, set up the audit table and log events from your model lifecycle hooks:

```rust
use rullst_orm::audit::{create_audit_table, log_audit_diff};

// Run once at startup (or in a migration)
create_audit_table().await?;

// Log only changed fields between two JSON snapshots
log_audit_diff("User", user.id, "updated", &old_json, &new_json).await?;
```

## Features

- **Dark Mode Native**: A stunning, premium dark interface out-of-the-box.
- **Audit Logs View**: Directly inspect `rullst_audits` to see revision history.
- **Zero Config**: No React/Vue compilation required — just serve the string.
- **Zero Dependencies**: The HTML is a `&'static str` baked at compile time.
