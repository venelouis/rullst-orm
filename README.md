# Rullst ORM 🌟

![Crates.io](https://img.shields.io/crates/v/rullst-orm?style=flat-square&color=orange)
![Downloads](https://img.shields.io/crates/d/rullst-orm?style=flat-square&color=blue)
![Docs.rs](https://img.shields.io/docsrs/rullst-orm?style=flat-square&color=blue)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
![Databases](https://img.shields.io/badge/Databases-PostgreSQL%20%7C%20MySQL%20%7C%20SQLite-lightgrey?style=flat-square)
![CI](https://github.com/venelouis/rullst-orm/actions/workflows/ci.yml/badge.svg)

An Active Record ORM for Rust.

Built on top of `sqlx` and procedural macros, **rullst-orm** brings a clean, fluent Active Record API to the Rust ecosystem. It supports **PostgreSQL**, **MySQL**, and **SQLite** through compile-time feature flags.

## 🚀 Why Rullst ORM?

In traditional Rust database handling, you write raw SQL, manage connection pools manually across every function, and bind variables repetitively. Rullst ORM solves this by abstracting the heavy lifting behind a single `#[derive(Orm)]` macro.

**Rullst ORM v4.0** includes:
- **Read/Write Connection Splitting** — automatic routing to read replicas.
- **Integrated Redis Caching** — speed up repeating queries with `.remember(ttl)`.
- **Query Chunking** — memory-safe large dataset processing.
- **Constrained Eager Loading** — fetch deep relationships without N+1 queries.
- **Global Lifecycle Observers** — intercept operations before/after they happen.
- **Subqueries & Advanced Joins** — multi-constraint `ON` clauses with binding safety.
- **Artisan Migrations CLI** — auto-generate, run, and roll back database schemas.
- **Dynamic Query Logging** — toggle STDOUT SQL logging at runtime.
- **Multi-Tenancy** — async-safe tenant isolation via task-local context.
- **Audit Logging** — automatic diff-based change trails.
- **Admin Dashboard** — built-in dark-mode web panel, zero dependencies.
- **API Resources** — transform models to JSON with a clean trait.
- **Collection Utilities** — `map`, `filter`, `chunk`, `implode`, and more on every `Vec<Model>`.

---

## 📚 Documentation & Planning

- **[Changelog](https://github.com/venelouis/rullst-orm/blob/main/CHANGELOG.md)**: Detailed release history.
- **[ISSUES](https://github.com/venelouis/rullst-orm/issues)**: Any issues? Please report.
- **[Spec](https://github.com/venelouis/rullst-orm/blob/main/docs/spec.md)**: Single Source of Truth for macros, API, and architecture.
- **[Getting Started](https://github.com/venelouis/rullst-orm/blob/main/docs/1-getting-started.md)**: Step-by-step first model and queries.
- **[Admin Panel](https://github.com/venelouis/rullst-orm/blob/main/docs/2-admin-panel.md)**: Serving the built-in dashboard.
- **[AI Agents & Automation](./AGENTS.md)**: Example prompts and agent context for contributors.

---

## 🛠️ Installation

```toml
[dependencies]
rullst-orm = { version = "4.0", features = ["postgres"] }
# or features = ["mysql"]
# or features = ["sqlite"]
tokio = { version = "1", features = ["full"] }
```

---

## 📖 Quick Start

```rust
use rullst_orm::Orm;

// 1. Define your model
#[derive(Debug, Clone, Orm)]
#[orm(table = "users")]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    #[orm(hidden)] // excluded from .to_json()
    pub password: String,
}

#[tokio::main]
async fn main() -> Result<(), rullst_orm::Error> {
    // 2. Initialize the global connection pool
    Orm::init("sqlite::memory:").await?;

    // 3. Create a new user
    let mut user = User {
        id: 0,
        name: "Vene Louis".to_string(),
        email: "vene@cosmos.com".to_string(),
        password: "secret".to_string(),
    };
    user.save().await?; // Runs INSERT and updates the ID automatically

    // 4. Update the user
    user.name = "John Doe".to_string();
    user.save().await?; // Detects id > 0, runs UPDATE automatically

    // 5. Fetch from database
    let found = User::query().find(1).await?;
    println!("Found: {:?}", found);

    // 6. Delete
    if let Some(u) = found {
        u.delete().await?;
    }

    Ok(())
}
```

---

## ✨ Query Builder API

The `#[derive(Orm)]` macro injects a full Query Builder into your model.

### 🔍 Active Record Methods

| Method | Description |
|---|---|
| `Model::query()` | Start a new Query Builder instance |
| `Model::query().find(id)` | Find a single record by Primary Key |
| `Model::query().first()` | First matching record (`LIMIT 1`) |
| `Model::query().get()` | All matching records as `Vec<Model>` |
| `model.save()` | `INSERT` if `id == 0`, else `UPDATE` |
| `model.delete()` | Delete the record (or soft-delete if `deleted_at` is present) |

### ⛓️ Query Filters (Chainable)

All values are automatically bound to prevent SQL Injection.

**AND Filters:**
- `.where_eq(column, value)` / `.where_not_eq(column, value)`
- `.where_gt(column, value)` / `.where_lt(column, value)` / `.where_gte` / `.where_lte`
- `.where_like(column, value)`
- `.where_null(column)` / `.where_not_null(column)`
- `.where_in(column, vec_of_values)` / `.where_not_in(column, vec_of_values)`
- `.where_between(column, min, max)` / `.where_not_between(column, min, max)`

**OR Filters:**
- `.or_where(column, value)` / `.or_where_not_eq(column, value)`
- `.or_where_like(column, value)` / `.or_where_not_null(column)`
- `.or_where_in(column, vec_of_values)` / `.or_where_gt(column, value)`

**Raw SQL:**
- `.where_raw(sql)` — raw SQL fragment
- `.bind(value)` — bind a typed value to the previous `?` placeholder

### 🔢 Sorting, Limits & Aggregation

- `.order_by(column)` / `.order_by_desc(column)`
- `.limit(n)` / `.offset(n)` — aliases: `.take(n)` / `.skip(n)`
- `.latest(column)` / `.oldest(column)`
- `.select_raw("col1, col2")` / `.group_by(column)`
- `.count().await?` → `i64`
- `.delete_all().await?` — delete all matching rows

### ⚡ Terminal Executors

- `.get().await?` → `Vec<Model>`
- `.first().await?` → `Option<Model>`
- `.find(id).await?` → `Option<Model>`
- `.paginate(page, per_page).await?` → `PaginationResult<Model>`

```rust
pub struct PaginationResult<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub per_page: usize,
    pub current_page: usize,
    pub last_page: usize,
}
```

---

## 🛡️ Raw Queries & SQL Injection Prevention

**Never** interpolate user input directly into `.where_raw()`. Always follow with `.bind()`:

```rust
// ❌ DANGEROUS — SQL Injection risk
let query = User::query().where_raw(&format!("email = '{}'", input_email));

// ✅ SECURE — parameterized binding
let query = User::query()
    .where_raw("email = ? AND status = ?")
    .bind(input_email)
    .bind("active");
```

---

## 🚀 Advanced Subqueries & Joins

### Constrained Joins

```rust
let posts = Post::query()
    .join_constrained("users", |join| {
        join.on("posts.user_id", "=", "users.id")
            .on_eq("users.active", true)
    })
    .where_eq("posts.status", "published")
    .get()
    .await?;
```

### Subqueries (`where_exists`)

```rust
let active_users = User::query()
    .where_exists(
        Post::query()
            .where_column("posts.user_id", "users.id")
            .where_eq("posts.status", "published")
    )
    .get()
    .await?;
```

---

## 🛡️ Global Lifecycle Observers

```rust
pub struct UserObserverImpl;

#[rullst_orm::async_trait]
impl UserObserver for UserObserverImpl {
    async fn saving(&self, model: &mut User) -> Result<(), rullst_orm::Error> {
        println!("About to save user: {}", model.name);
        Ok(())
    }
}

// Register globally once:
User::observe(Arc::new(UserObserverImpl));
```

**Supported events**: `saving`, `saved`, `creating`, `created`, `updating`, `updated`, `deleting`, `deleted`.

---

## 🏢 Enterprise Scaling

### Read/Write Connection Splitting

```rust
Orm::init_with_replicas(
    "postgres://primary_db_url",
    vec![
        "postgres://replica_1_url".to_string(),
        "postgres://replica_2_url".to_string(),
    ],
).await?;

// SELECTs go to replicas automatically (round-robin)
let users = User::query().get().await?;

// INSERT/UPDATE/DELETE go to primary automatically
let mut user = User::query().find(1).await?.unwrap();
user.name = "Updated".to_string();
user.save().await?;
```

### Redis Caching Layer

```rust
Orm::init_redis("redis://127.0.0.1/").await?;

let active_users = User::query()
    .where_eq("status", "active")
    .remember(3600) // cache for 1 hour
    .get()
    .await?;
```

### Query Chunking

```rust
User::query()
    .where_eq("status", "active")
    .chunk(1000, |batch| Box::pin(async move {
        for user in &batch {
            println!("Processing: {}", user.name);
        }
        Ok(())
    }))
    .await?;
```

---

## 🏢 Multi-Tenancy

```rust
use rullst_orm::tenant::{with_tenant, get_tenant_id};

with_tenant("acme_corp", async {
    let tenant = get_tenant_id(); // Some(RullstValue::String("acme_corp"))
    // All ORM queries run inside this scope
}).await;
```

---

## 📋 Audit Logging

```rust
use rullst_orm::audit::{create_audit_table, log_audit_diff};

create_audit_table().await?;
log_audit_diff("User", user.id, "updated", &old_json, &new_json).await?;
```

---

## 🖥️ Admin Dashboard

```rust
use rullst_orm::admin::dashboard_html;

// Axum example:
let app = Router::new()
    .route("/admin", get(|| async { Html(dashboard_html()) }));
```

---

## 🐘 Artisan CLI (Migrations & Seeding)

```rust
// In your CLI entry point:
rullst_orm::schema::run_artisan(std::env::args().collect(), vec![
    // Seeders here
]).await;
```

**Commands:**
- `make:migration create_users_table` — scaffold a `.rs` migration file
- `migrate` — execute pending migrations
- `migrate:rollback` — undo the previous batch
- `db:seed` — run database seeders

---

## 🔎 Query Debug Logging

```rust
Orm::enable_query_log();
// All SQL, parameters, limits, and offsets print to STDOUT
Orm::disable_query_log();
```

---

## ⚙️ Compile-Time Field Methods

The macro inspects your struct at compile time and generates typed methods per field. For a `name: String` field, you automatically get:

- `.where_name(value)`
- `.or_where_name(value)`
- `.where_not_name(value)`
- `.order_by_name()` / `.order_by_name_desc()`
