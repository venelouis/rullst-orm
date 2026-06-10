<div align="center">
  <h1>Rullst ORM 🌟</h1>
  <p><strong>A beautiful, type-safe, Active Record ORM for Rust.</strong></p>

  <p>
    <a href="https://crates.io/crates/rullst-orm"><img src="https://img.shields.io/crates/v/rullst-orm?style=flat-square&color=orange" alt="Crates.io" /></a>
    <a href="https://docs.rs/rullst-orm"><img src="https://img.shields.io/docsrs/rullst-orm?style=flat-square&color=blue" alt="Docs.rs" /></a>
    <img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="License: MIT" />
    <img src="https://img.shields.io/badge/Databases-PostgreSQL%20%7C%20MySQL%20%7C%20SQLite-lightgrey?style=flat-square" alt="Databases" />
  </p>
</div>

🚀 **[Visit the Official Website & Documentation Hub](https://venelouis.github.io/rullst-orm/)** 🚀

Built on top of `sqlx` and procedural macros, **Rullst ORM** brings the delightful, fluent syntax of Active Record frameworks (like Laravel's Eloquent) directly to the high-performance Rust ecosystem.

## 🚀 Why Rullst ORM?

In traditional Rust database handling, you have to write raw SQL queries, manage connection pools manually, and bind variables repetitively. Rullst ORM abstracts the heavy lifting behind a single `#[derive(Orm)]` macro, generating hundreds of safe, chainable query methods at compile time.

**Key Features:**
- **Zero-Boilerplate CRUD**: Insert, update, delete, and find records instantly.
- **Fluent Query Builder**: Chain `.where_eq()`, `.limit()`, and `.order_by()` effortlessly.
- **Eager Loading**: Solve N+1 problems with robust `has_many`, `belongs_to`, and `morph_many` relations.
- **Built-in Multi-Tenancy**: Automatically scope all queries by tenant ID.
- **Automated Audit Logs**: Track `old_values` and `new_values` history natively.
- **Scout Search**: Seamlessly sync models to full-text search engines.
- **Enterprise Ready**: Read/write replica splitting, query chunking, and Redis caching built-in.

---

## 🛠️ Quick Start

### Installation

Add the library to your `Cargo.toml`:

```bash
cargo add rullst-orm
cargo add tokio -F full
```

### Zero-to-Hero Example

```rust
use rullst_orm::{Orm, FromRow};

// 1. Just add the Orm macro to your struct!
#[derive(Debug, Clone, FromRow, Orm)]
pub struct User {
    pub id: i32, // ID = 0 means it hasn't been saved yet
    pub name: String,
    pub email: String,
    #[orm(hidden)] // Won't be exposed in JSON responses
    pub password: String,
}

#[tokio::main]
async fn main() -> Result<(), rullst_orm::Error> {
    // 2. Initialize the connection pool (Supports SQLite, Postgres, MySQL)
    Orm::init("sqlite::memory:").await?;

    // 3. Create a new user magically
    let mut user = User {
        id: 0,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        password: "secret_password".to_string(),
    };
    
    user.save().await?; // Runs INSERT and hydrates the ID automatically!

    // 4. Fluent Queries
    let active_users = User::query()
        .where_like("email", "%@example.com")
        .order_by_desc("id")
        .limit(10)
        .get()
        .await?;

    println!("Found users: {:?}", active_users);

    Ok(())
}
```

---

## 📚 Documentation

The documentation is kept lean and straight to the point. Dive into the modules below to master Rullst ORM:

- [1. Basics & Query Builder](docs/1-basics.md): Connecting to the DB, filtering, sorting, and raw bindings.
- [2. Relationships](docs/2-relationships.md): Has Many, Belongs To, Polymorphic relations, and Eager Loading.
- [3. Advanced Features](docs/3-advanced-features.md): Multi-Tenancy, Audit Trails, Redis Caching, and Observers.
- [4. Migrations & Schema](docs/4-migrations-schema.md): Building tables programmatically and using the Artisan CLI.

---

## 🛡️ Security

Rullst ORM employs rigorous defenses against **SQL Injection**. All dynamic builder methods (like `.where_eq()`) automatically escape values using `sqlx` prepared statement bindings (`$1` or `?`). Raw queries (`.where_raw()`) actively force developers to provide an array of bindings directly in the function signature. Furthermore, all structural identifiers (table and column names) are validated strictly at runtime against a whitelist regex.

## 📄 License
This project is licensed under the [MIT License](LICENSE).
