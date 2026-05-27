# Rust Eloquent - Version 2.0.0 Roadmap

This document outlines the planned breaking changes and architectural upgrades for the next major release (`v2.0.0`). These changes were intentionally deferred from the `v1.x` branch to preserve backward compatibility and maintain the simplistic, lifetime-free API.

## 1. ⚡ Zero-Copy Query Builder (String Optimization)

**Current State (v1.x):**
The `QueryBuilder` allocates new `String` objects on the heap using `format!` for every condition (e.g., `where_eq`, `join`, `order_by`). This was done to keep the API simple and avoid polluting the builder and `ActiveRecord` implementation with generic lifetimes (`<'a>`).

**Proposed Change (v2.0):**
Refactor the internal `wheres`, `joins`, and `selects` collections to use `std::borrow::Cow<'a, str>`.
- This will completely eliminate heap allocations for static column names and SQL fragments.
- **Breaking Change:** The `QueryBuilder` struct will require a lifetime parameter `QueryBuilder<'a>`. All functions returning or chaining the builder will need to declare this lifetime, cascading into the asynchronous `Future` bounds of `ActiveRecord` methods.

## 2. 🛡️ Strict SQL Typing (Removing `AnyPool`)

**Current State (v1.x):**
The library uses `sqlx::AnyPool` and a custom generic enum (`EloquentValue`) to map types dynamically at runtime. This allows the ORM to connect to PostgreSQL, MySQL, and SQLite seamlessly without changing the Rust codebase. However, it sacrifices Rust's powerful compile-time SQL verification.

**Proposed Change (v2.0):**
Introduce an optional "Strict Mode" (e.g., `#[eloquent(strict(postgres))]`).
- When enabled, the ORM will use `sqlx::query!` macros to validate SQL queries against the actual database schema at compile time.
- **Breaking Change:** Developers will lose dynamic database swapping for models opted into strict mode. The underlying pool will change from `AnyPool` to specific pools like `PgPool` or `MySqlPool`.

## 3. 🧹 Automated Resource Cleanup (Subquery Scopes)

**Current State (v1.x):**
Subqueries and raw scope injections do not automatically drop their memory footprints until the parent query completes execution.

**Proposed Change (v2.0):**
Implement custom `Drop` traits or an explicit arena allocator for complex query chains to reduce the maximum memory footprint during large `EXISTS` subquery resolutions.
