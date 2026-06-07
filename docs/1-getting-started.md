# Getting Started

## Installation

Add `rullst-orm` to your `Cargo.toml`:

```toml
[dependencies]
rullst-orm = { version = "4.0", features = ["postgres"] }
# Or for SQLite:
# rullst-orm = { version = "4.0", features = ["sqlite"] }
# Or for MySQL:
# rullst-orm = { version = "4.0", features = ["mysql"] }
```

## Initialize the Connection

Call `Orm::init()` once at application startup before making any queries:

```rust
use rullst_orm::Orm;

#[tokio::main]
async fn main() -> Result<(), rullst_orm::Error> {
    // Single database
    Orm::init("postgres://user:pass@localhost/mydb").await?;

    // Or with read replicas
    Orm::init_with_replicas(
        "postgres://primary/mydb",
        vec!["postgres://replica-1/mydb", "postgres://replica-2/mydb"],
    ).await?;

    Ok(())
}
```

## Your First Model

Define a standard Rust struct and add `#[derive(Orm)]`:

```rust
use rullst_orm::Orm;

#[derive(Orm, Clone, Debug)]
#[orm(table = "users")]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>, // enables soft-delete when present
}
```

## Basic Queries

Rullst ORM generates a powerful, type-safe query builder for every model.

```rust
// Fetch all users
let users = User::query().get().await?;

// Find by ID
let user = User::query().find(1).await?;

// First matching record
let user = User::query().first().await?;

// Advanced where clauses
let admins = User::query()
    .where_eq("role", "admin")
    .where_gt("age", 18)
    .order_by_desc("created_at")
    .get()
    .await?;

// Paginate results
let page = User::query()
    .where_eq("active", true)
    .paginate(1, 15)
    .await?;
// page.data -> Vec<User>
// page.total, page.last_page, page.current_page, page.per_page
```

## Save & Delete

```rust
// Create or update
let mut user = User { id: 0, name: "Alice".into(), email: "alice@example.com".into(), ... };
user.save().await?;

// Delete
user.delete().await?;
// Soft-delete (if deleted_at field is present): sets deleted_at instead of removing the row
```

## Transactions

```rust
let mut tx = Orm::begin_transaction().await?;
// ... perform operations inside tx ...
tx.commit().await?;
// tx.rollback().await?;
```
