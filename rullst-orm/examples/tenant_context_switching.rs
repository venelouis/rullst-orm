//! Tenant scoping + per-query opt-out.
//!
//! Demonstrates the two-layer tenant API:
//!
//! 1. `rullst_orm::with_tenant(t)` — request-scoped default, set
//!    once at the HTTP boundary.
//! 2. `QueryBuilder::without_tenant()` — drop the WHERE for one
//!    query, even while a `with_tenant` scope is active.
//!
//! Run with:
//!   cargo run --example tenant_context_switching

use rullst_orm::{with_tenant, Orm};

#[derive(Clone, Debug, Default, rullst_orm::Orm, rullst_orm::FromRow)]
#[orm(table = "products", tenant_column = "tenant_id")]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub tenant_id: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = std::fs::remove_file("test_tenant_switch.db");
    std::fs::File::create("test_tenant_switch.db").unwrap();
    Orm::init("sqlite:test_tenant_switch.db").await?;
    let pool = Orm::pool();

    sqlx::query(
        "CREATE TABLE products (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, tenant_id TEXT)",
    )
    .execute(pool)
    .await?;

    // Seed two tenants with two products each.
    for (name, tenant) in [
        ("Apple iPhone", "tenant_uuid_1"),
        ("Apple iPad", "tenant_uuid_1"),
        ("Samsung Galaxy", "tenant_uuid_2"),
        ("Samsung Watch", "tenant_uuid_2"),
    ] {
        with_tenant(tenant, async {
            let mut p = Product {
                name: name.to_string(),
                ..Default::default()
            };
            p.save().await.unwrap();
        })
        .await;
    }

    // -----------------------------------------------------------------
    // 1) Normal scope: every query is filtered by the active tenant.
    // -----------------------------------------------------------------
    let t1_count = with_tenant("tenant_uuid_1", async {
        Product::all().await.map(|v| v.len()).unwrap()
    })
    .await;
    println!("[scope: t1] Product::all() = {} rows", t1_count);

    // -----------------------------------------------------------------
    // 2) Without leaving the t1 scope, peek at ALL products using
    //    `without_tenant()`. This is the "super admin can read
    //    across tenants" use case.
    // -----------------------------------------------------------------
    let t1_count_with_skip = with_tenant("tenant_uuid_1", async {
        Product::query()
            .without_tenant()
            .get()
            .await
            .map(|v| v.len())
            .unwrap()
    })
    .await;
    println!(
        "[scope: t1, query.without_tenant()] = {} rows (sees every tenant)",
        t1_count_with_skip
    );

    Ok(())
}
