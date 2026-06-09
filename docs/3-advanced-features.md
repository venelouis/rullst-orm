# 3. Advanced Features

Rullst ORM packs an arsenal of enterprise-grade features intended for SaaS scaling, cache management, and strict compliance.

---

## 🏢 Multi-Tenancy

Rullst seamlessly supports single-database Multi-Tenancy out-of-the-box using asynchronous task-local scoping (`tokio::task_local!`).

1. Flag the column in your macro:
```rust
#[derive(Debug, Clone, FromRow, Orm)]
#[orm(tenant_column = "company_id")] // <--- Enables Multi-Tenancy
pub struct User {
    pub id: i32,
    pub company_id: String,
    pub name: String,
}
```

2. Wrap your operations using `with_tenant`. The ORM will automatically apply `WHERE company_id = ?` to all reads, updates, and deletes, and inject the `company_id` onto all new inserts.

```rust
use rullst_orm::tenant::with_tenant;

let results = with_tenant("company_A1", async {
    // This query magically becomes: 
    // SELECT * FROM users WHERE company_id = 'company_A1'
    let users = User::all().await.unwrap(); 

    // This insert magically populates company_id automatically
    let mut new_user = User { id: 0, company_id: "".to_string(), name: "Bob".to_string() };
    new_user.save().await.unwrap();
}).await;
```

---

## 🛡️ Audit Trails

Automatically track a history of row modifications for compliance and debugging. Just add `#[orm(auditable)]`.

```rust
#[derive(Debug, Clone, FromRow, Orm)]
#[orm(auditable)] // <--- Intercepts and logs state diffs
pub struct Post {
    pub id: i32,
    pub title: String,
    pub content: String,
}
```
Whenever a `Post` is saved or deleted, Rullst compares the previous database state with the newly provided struct. If fields changed, it creates a JSON diff (`old_values` / `new_values`) and saves it inside the `rullst_audits` table.

*(Make sure to run the `make:audit` migration to instantiate the table first!)*

---

## 🔎 Scout Full-Text Search

If you enable Scout on a model, Rullst will automatically index it into Meilisearch/Algolia via API, or default to a powerful internal `LIKE` engine if no external backend is attached.

```rust
#[derive(Debug, Clone, FromRow, Orm)]
#[orm(scout)]
pub struct Article {
    pub id: i32,
    pub body: String,
}

// Perform a full-text search safely!
let hits = Article::search("Rust language").get().await?;
```

---

## 🚀 Read/Write Splitting & Caching

If you operate on distributed databases or require extreme speeds, Rullst splits traffic natively.

### Replicas
Initialize read replicas alongside your primary database. Rullst automatically sends `SELECT` queries in a round-robin format to replicas, keeping `INSERT/UPDATE/DELETE` on the primary instance.
```rust
Orm::init("postgres://master").await?;
Orm::init_replicas(&["postgres://replica_1", "postgres://replica_2"]).await?;
```

### Redis `remember()`
If you compiled the crate with the `redis` feature, you can cache any expensive SQL query instantly:

```rust
Orm::init_redis("redis://127.0.0.1/").await?;

// First query hits Postgres. All queries for the next 3600 seconds hit Redis!
let stats = Analytics::query()
    .remember(3600)
    .get()
    .await?;
```
