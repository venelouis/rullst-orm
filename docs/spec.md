# rullst-orm Specification 📄
### *"The Single Source of Truth (SST) for ORM Architecture & Macros"*

This document is the **Single Source of Truth (SST)** for the **rullst-orm ORM**. It specifies the exact macros, attributes, query builders, and database connection architectures available in `rullst-orm`.

> [!IMPORTANT]
> **AI Alignment Instruction:**
> Whenever updating, refactoring, or generating code for applications using rullst-orm, **always** refer to this specification as the baseline. Do not invent or assume macro parameters or query patterns outside of this document.

> [!NOTE]
> **Version:** This spec reflects `rullst-orm v4.0.3`.
> All public APIs return `Result<T, rullst_orm::Error>` (not `sqlx::Error`). The underlying `sqlx` crate is an internal implementation detail and is not re-exported.

---

## 📂 1. Model Definition & Macro Attributes

All Active Record entities are defined as Rust structs deriving `rullst_orm::Orm`:

```rust
use rullst_orm::{Orm, RullstModel};

#[derive(Debug, Clone, Orm)]
#[orm(
    table = "table_name",            // Map to custom database table (defaults to lowercase plural struct name)
    global_scope = "my_scope",      // Apply query filter scope globally to all SELECTs
    before_save = "saving_hook",    // Method called before saving (creating or updating)
    after_save = "saved_hook",      // Method called after saving
    before_delete = "deleting_hook",// Method called before deletion
    after_delete = "deleted_hook",  // Method called after deletion
    after_fetch = "loaded_hook"     // Method called after fetching records
)]
pub struct BlogPost {
    pub id: i32,
    pub title: String,

    #[orm(hidden)]                  // Skip this field during JSON serialization
    pub secret_token: String,

    pub created_at: String,         // Automatic auto-timestamp
    pub updated_at: String,         // Automatic auto-timestamp
    pub deleted_at: Option<String>, // Triggers soft-delete mode when present in struct
}
```

---

## 🔗 2. Declarative Relationships

Declare relationships directly on model fields using custom `#[orm(...)]` attributes. The derive macro generates both direct fetching futures and eager loading hooks.

### 2.1. One-to-Many (`has_many`)
One record owns multiple child records.
```rust
#[orm(has_many = "Comment", foreign_key = "post_id", local_key = "id")]
pub comments: Option<Vec<Comment>>,
```

### 2.2. One-to-One (`has_one`)
One record owns exactly one child record.
```rust
#[orm(has_one = "Profile", foreign_key = "user_id", local_key = "id")]
pub profile: Option<Profile>,
```

### 2.3. Inverse Relationship (`belongs_to`)
A child record belongs to a parent record.
```rust
#[orm(belongs_to = "User", foreign_key = "user_id", related_key = "id")]
pub user: Option<User>,
```

### 2.4. Many-to-Many (`belongs_to_many`)
Records linked through an intermediate pivot table.
```rust
#[orm(belongs_to_many = "Role", pivot_table = "role_user", foreign_key = "user_id", related_key = "role_id", local_key = "id")]
pub roles: Option<Vec<Role>>,
```

### 2.5. Polymorphic One-to-Many (`morph_many`)
A target model belongs to more than one type of model on a single association.
```rust
#[orm(morph_many = "Comment", name = "commentable", local_key = "id")]
pub comments: Option<Vec<Comment>>,
```
*Creates column checks for `<name>_type` and `<name>_id` on the target table (e.g. `commentable_type = "BlogPost"` and `commentable_id = blog_post.id`).*

### 2.6. Polymorphic One-to-One (`morph_one`)
```rust
#[orm(morph_one = "Image", name = "imageable", local_key = "id")]
pub image: Option<Image>,
```

---

## ⚡ 3. Fluent Query Builder API

`Model::query()` returns a compiled `ModelQueryBuilder` that supports chainable queries, subqueries, and replica routing.

### 3.1. Conditional Comparisons
* `.where_eq(column, value)`
* `.where_not_eq(column, value)`
* `.where_gt(column, value)`
* `.where_lt(column, value)`
* `.where_gte(column, value)`
* `.where_lte(column, value)`
* `.where_like(column, value)`
* `.where_null(column)`
* `.where_not_null(column)`
* `.where_in(column, Vec<values>)`
* `.where_not_in(column, Vec<values>)`
* `.where_between(column, min, max)`
* `.where_not_between(column, min, max)`
* `.or_where(column, value)`
* `.or_where_gt(column, value)`
* `.or_where_lt(column, value)`
* `.or_where_like(column, value)`
* `.or_where_not_eq(column, value)`
* `.or_where_not_null(column)`
* `.or_where_in(column, Vec<values>)`
* `.where_raw(sql)` — raw SQL fragment (use with `.bind()` for safety)
* `.bind(value)` — binds a typed value to the most recent `.where_raw()` placeholder

> [!CAUTION]
> **Never interpolate user input directly into `.where_raw()`.** Always follow it with `.bind(value)` to use parameterized queries and prevent SQL injection.

### 3.2. Scopes, Sorting & Limits
* `.take(limit: usize)` / `.limit(limit: usize)`
* `.skip(offset: usize)` / `.offset(offset: usize)`
* `.latest(column)` / `.oldest(column)`
* `.order_by(column)` / `.order_by_desc(column)`

### 3.3. Joins & Aggregates
* `.join(table, first, operator, second)`
* `.left_join(table, first, operator, second)`
* `.join_constrained(table, |join_clause| ...)` — closure receives a `&mut JoinClause`
* `.where_exists(subquery)`

**`JoinClause` methods** (used inside `.join_constrained`):
* `.on(first, operator, second)` — column-to-column join condition; validates identifiers
* `.on_eq(column, value)` — binds a typed value directly to a join condition

### 3.4. Cache Integration
* `.remember(seconds: u32)`: Automatically cache query results in Redis for the specified TTL.

### 3.5. Eager Loading (N+1 Prevention)
* `.with_comments()`: Load comments relation.
* `.with_comments_constrained(|q| q.where_eq("approved", true))`: Load relation applying filter.

---

## 📈 4. Pagination & Results

* `.get().await` → `Result<Vec<Model>, rullst_orm::Error>`
* `.first().await` → `Result<Option<Model>, rullst_orm::Error>`
* `.find(id).await` → `Result<Model, rullst_orm::Error>`
* `.paginate(page: usize, per_page: usize).await` → `Result<PaginationResult<Model>, rullst_orm::Error>`

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

## 🏭 5. Connection Pools & Replication Splitting

Agnostic database connection management split cleanly between writes (primary node) and reads (load-balanced round-robin replicas).

* **Single connection:**
  ```rust
  Orm::init("sqlite://rullst.db").await?;
  ```
* **Primary / Replica routing:**
  ```rust
  Orm::init_with_replicas(
      "postgres://primary-db.host/prod",
      vec!["postgres://replica-1.host/prod", "postgres://replica-2.host/prod"]
  ).await?;
  ```
* **Redis cache initialization:**
  ```rust
  Orm::init_redis("redis://127.0.0.1/").await?;
  ```
* **Query splitting:**
  * All builder execution methods like `.get()`, `.first()`, `.paginate()` dynamically fetch from `Orm::read_pool()`.
  * All mutative operations like `.save()`, `.delete()`, `.begin_transaction()` dynamically fetch from `Orm::pool()`.
* **Query logging:**
  * `Orm::enable_query_log()` / `Orm::disable_query_log()` — toggle SQL statement logging to stdout.

---

## 🧪 6. Factories, Observers & Seeders

### 6.1. Entity Factories
Fluent generation of fake testing data:
```rust
let users = User::factory(|| User {
    id: 0,
    name: "Fake User".to_string(),
})
.count(5)
.create()
.await?;
```

### 6.2. Observers
Attach lifecycle listeners externally:
```rust
#[rullst_orm::async_trait]
pub trait UserObserver: Send + Sync {
    async fn creating(&self, model: &mut User) -> Result<(), rullst_orm::Error>;
    async fn created(&self, model: &User) -> Result<(), rullst_orm::Error>;
}
```

### 6.3. Seeders
Standard populate traits:
```rust
#[rullst_orm::async_trait]
impl Seeder for DatabaseSeeder {
    async fn run(&self) -> Result<(), rullst_orm::Error> {
        User::factory(|| User { ... }).count(10).create().await?;
        Ok(())
    }
}
```

---

## 🗄️ 7. Schema Builder & Migrations

Define database tables programmatically using the fluent `Blueprint` API:

```rust
use rullst_orm::schema::{Schema, Blueprint};

Schema::create("users", |t: &mut Blueprint| {
    t.id();
    t.string("name").not_null();
    t.string("email").not_null();
    t.integer("age").nullable().default("0");
    t.boolean("active").not_null();
    t.timestamps();
    t.soft_deletes();
}).await?;

Schema::drop_if_exists("users").await?;
```

**`Blueprint` column methods:**
| Method | SQL type |
|---|---|
| `.id()` | `INTEGER PRIMARY KEY AUTOINCREMENT` |
| `.string(name)` | `VARCHAR(255)` |
| `.integer(name)` | `INTEGER` |
| `.float(name)` | `REAL` / `FLOAT` |
| `.boolean(name)` | `BOOLEAN` |
| `.timestamps()` | adds `created_at` + `updated_at` |
| `.soft_deletes()` | adds nullable `deleted_at` |

**Column modifiers** (chainable on all column types):
* `.not_null()` / `.nullable()` / `.default(val)` / `.primary()`

**Migration trait:**
```rust
#[async_trait::async_trait]
impl rullst_orm::schema::Migration for MyMigration {
    fn name(&self) -> &str { "2024_create_users_table" }
    async fn up(&self) -> Result<(), rullst_orm::Error> { ... }
    async fn down(&self) -> Result<(), rullst_orm::Error> { ... }
}
```

---

## 🔍 8. Search Engine Integration (Scout)

Register a custom search backend that implements the `SearchEngine` trait:

```rust
use rullst_orm::scout::{SearchEngine, set_search_engine, get_search_engine};

#[async_trait::async_trait]
impl SearchEngine for MeilisearchEngine {
    async fn update(&self, table: &str, id: i32, payload: serde_json::Value) -> Result<(), rullst_orm::Error>;
    async fn delete(&self, table: &str, id: i32) -> Result<(), rullst_orm::Error>;
    async fn search(&self, table: &str, query: &str) -> Result<Vec<i32>, rullst_orm::Error>;
}

set_search_engine(Box::new(MeilisearchEngine::new()));
let engine = get_search_engine(); // -> Option<&'static dyn SearchEngine>
```

---

## 🏢 9. Multi-Tenancy

Scoped tenant context propagated via Tokio task-local storage:

```rust
use rullst_orm::tenant::{with_tenant, get_tenant_id};

// Set tenant for the duration of an async block
with_tenant("acme_corp", async {
    let tenant = get_tenant_id(); // -> Option<RullstValue>
    // All ORM queries inside this scope can inspect the active tenant
}).await;
```

---

## 📋 10. Audit Logging

Automatic audit trail for model mutations:

```rust
use rullst_orm::audit::{log_audit, log_audit_diff, create_audit_table, compute_diff};

// Create the audit table if it doesn't exist
create_audit_table().await?;

// Log a raw event
log_audit("User", 42, "updated", old_json, new_json).await?;

// Log only changed fields (computes diff automatically)
log_audit_diff("User", 42, "updated", old_json_str, new_json_str).await?;

// Compute diff without writing to DB (pure function, useful for testing)
let (old_diff, new_diff) = compute_diff(old_json_str, new_json_str);
```

```rust
pub struct AuditLog {
    pub id: i32,
    pub model_type: String,
    pub model_id: i32,
    pub event: String,
    pub old_values: Option<String>,
    pub new_values: Option<String>,
    pub created_at: Option<String>,
}
```

---

## 📦 11. Collections (`RullstCollection`)

An extension trait implemented on `Vec<T>` bringing functional collection methods:

```rust
use rullst_orm::collection::RullstCollection;

let users: Vec<User> = ...;

// Transform
let names: Vec<String> = users.map(|u| u.name.clone());

// Filter
let admins: Vec<User> = users.filter(|u| u.is_admin);

// Group by key
let by_id: HashMap<i32, User> = users.key_by(|u| u.id);

// Split into chunks
let pages: Vec<Vec<User>> = users.chunk(10);

// Join to string
let csv: String = users.implode(", ", |u| u.name.clone());

// Aggregates
let total_age: i32 = users.sum_by(|u| u.age);
let oldest: Option<&User> = users.max_by_key(|u| u.age);
let youngest: Option<&User> = users.min_by_key(|u| u.age);
```

---

## 🌐 12. API Resources

Transform models into JSON for API responses:

```rust
use rullst_orm::resource::{ApiResource, JsonResource, ResourceCollection};

impl ApiResource for User {
    fn to_array(&self) -> serde_json::Value {
        serde_json::json!({ "id": self.id, "name": self.name })
    }
}

// Single resource
let json = JsonResource::new(&user).resolve(); // -> serde_json::Value

// Collection resource
let json = ResourceCollection::new(&users).resolve(); // -> serde_json::Value (array)
```

---

## 🖥️ 13. Admin Dashboard

Built-in dark-mode admin panel HTML (for embedding into any web framework):

```rust
use rullst_orm::admin::dashboard_html;

let html: &'static str = dashboard_html();
// Return this as the HTTP response body for your /admin route
```

---

## ⚠️ 14. Error Handling

All public APIs return `Result<T, rullst_orm::Error>`. The error variants are:

```rust
pub enum Error {
    Database(String),    // Wraps sqlx errors transparently
    Internal(String),    // Framework-level failures (e.g. pool not initialized)
    Validation(String),  // Invalid identifiers, rejected query parameters
    NotFound,            // Record not found
}
```

> [!WARNING]
> `sqlx::Error` is **not** re-exported and must not be used directly in application code. Always match on `rullst_orm::Error` variants.
