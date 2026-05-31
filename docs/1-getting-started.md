# Getting Started

## Installation

Add `rullst-orm` to your `Cargo.toml`:

```bash
cargo add rullst-orm --features "postgres"
```

## Your First Model

Simply define your standard Rust struct and slap `#[derive(Orm)]` on it:

```rust
use rullst_orm::Orm;

#[derive(Orm, Clone, Debug)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    
    #[orm(created_at)]
    pub created_at: Option<String>,
}
```

## Basic Queries

Rullst ORM generates powerful, type-safe queries on the fly.

```rust
// Fetch all users
let users = User::all().await?;

// Find by ID
let user = User::find(1).await?;

// Advanced Where Clauses
let admins = User::where_col("role", "admin")
    .where_col("age", ">", 18)
    .order_by("created_at", "DESC")
    .get()
    .await?;
```
