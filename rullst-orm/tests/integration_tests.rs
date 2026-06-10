// Database integration tests for rullst-orm.
//
// Design constraints:
//   - `Orm::init()` stores the pool in a global `OnceLock` — calling it twice
//     in the same process panics.  All scenarios therefore live inside a single
//     `#[tokio::test]` and share the one pool initialised at the top.
//   - Each logical scenario uses a uniquely-named table so tests never
//     interfere with each other even if the order of execution changes.
//   - SQLite (file-based, `?mode=rwc`) is used so no external server is
//     needed.  The file is deleted before and after the suite.

#![cfg(not(any(feature = "strict-postgres", feature = "strict-mysql")))]

use rullst_orm::schema::{Blueprint, Schema};
use rullst_orm::types::Json;
use rullst_orm::{FromRow, Orm};
use serde::{Deserialize, Serialize};

// ── shared JSON payload type ───────────────────────────────────────────────
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Payload {
    value: String,
}

// ── model: users ──────────────────────────────────────────────────────────
#[derive(Debug, Clone, FromRow, rullst_orm::Orm)]
#[orm(table = "it_users")]
struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

// ── model: posts (belongs_to User) ────────────────────────────────────────
#[derive(Debug, Clone, FromRow, rullst_orm::Orm)]
#[orm(table = "it_posts")]
struct Post {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
}

// ── model: json carrier ───────────────────────────────────────────────────
#[derive(Debug, Clone, FromRow, rullst_orm::Orm)]
#[orm(table = "it_json_records")]
struct JsonRecord {
    pub id: i32,
    pub data: Json<Payload>,
}

// ── database path ─────────────────────────────────────────────────────────
const DB_FILE: &str = "it_suite.db";

// ══════════════════════════════════════════════════════════════════════════
// Main integration test — all scenarios run sequentially in one Tokio task
// so the global pool is initialised exactly once.
// ══════════════════════════════════════════════════════════════════════════
#[tokio::test]
async fn integration_suite() {
    // ── 0. Setup ──────────────────────────────────────────────────────────
    let _ = std::fs::remove_file(DB_FILE);
    Orm::init(&format!("sqlite:{}?mode=rwc", DB_FILE))
        .await
        .expect("Orm::init should succeed");

    scenario_crud().await;
    scenario_soft_delete().await;
    scenario_transactions().await;
    scenario_json_column().await;
    scenario_bulk_operations().await;
    scenario_schema_lifecycle().await;
    scenario_audit().await;
    scenario_query_result_ext().await;

    // ── cleanup ───────────────────────────────────────────────────────────
    let _ = std::fs::remove_file(DB_FILE);
}

// ── Scenario 1: basic CRUD ────────────────────────────────────────────────
async fn scenario_crud() {
    Schema::create("it_users", |t: &mut Blueprint| {
        t.id();
        t.string("name").not_null();
        t.string("email").not_null();
    })
    .await
    .expect("create it_users");

    // INSERT
    let mut user = User {
        id: 0,
        name: "Alice".into(),
        email: "alice@example.com".into(),
    };
    user.save().await.expect("save new user");
    assert!(user.id > 0, "id must be assigned after save");

    // SELECT by PK
    let found = User::find(user.id)
        .await
        .expect("find")
        .expect("user exists");
    assert_eq!(found.name, "Alice");
    assert_eq!(found.email, "alice@example.com");

    // UPDATE
    user.name = "Alice Updated".into();
    user.save().await.expect("update user");
    let updated = User::find(user.id)
        .await
        .expect("find updated")
        .expect("exists");
    assert_eq!(updated.name, "Alice Updated");

    // INSERT second record
    let mut user2 = User {
        id: 0,
        name: "Bob".into(),
        email: "bob@example.com".into(),
    };
    user2.save().await.expect("save Bob");

    // SELECT all
    let all = User::all().await.expect("all users");
    assert_eq!(all.len(), 2, "expected 2 users");

    // WHERE clause
    let found_bob = User::query()
        .where_eq("name", "Bob")
        .first()
        .await
        .expect("query")
        .expect("Bob exists");
    assert_eq!(found_bob.email, "bob@example.com");

    // COUNT
    let count = User::query().count().await.expect("count");
    assert_eq!(count, 2);

    // DELETE
    user.delete().await.expect("delete user");
    let after_delete = User::find(user.id).await.expect("find after delete");
    assert!(after_delete.is_none(), "deleted user should not be found");

    let count_after = User::query().count().await.expect("count after delete");
    assert_eq!(count_after, 1);

    Schema::drop_if_exists("it_users")
        .await
        .expect("drop it_users");
}

// ── Scenario 2: soft deletes ──────────────────────────────────────────────
#[derive(Debug, Clone, FromRow, rullst_orm::Orm)]
#[orm(table = "it_soft_users")]
struct SoftUser {
    pub id: i32,
    pub name: String,
    pub deleted_at: Option<String>,
}

async fn scenario_soft_delete() {
    Schema::create("it_soft_users", |t: &mut Blueprint| {
        t.id();
        t.string("name").not_null();
        t.soft_deletes();
    })
    .await
    .expect("create it_soft_users");

    let mut u = SoftUser {
        id: 0,
        name: "SoftAlice".into(),
        deleted_at: None,
    };
    u.save().await.expect("save SoftAlice");

    // soft-delete
    u.delete().await.expect("soft delete");

    // record still exists in DB but deleted_at is set
    let pool = Orm::pool();
    let row: Option<(i32, Option<String>)> =
        sqlx::query_as("SELECT id, deleted_at FROM it_soft_users WHERE id = ?")
            .bind(u.id)
            .fetch_optional(pool)
            .await
            .expect("raw query");

    let (_, deleted_at) = row.expect("row must exist");
    assert!(
        deleted_at.is_some(),
        "deleted_at must be set after soft delete"
    );

    // restore
    u.restore().await.expect("restore");

    let row2: Option<(i32, Option<String>)> =
        sqlx::query_as("SELECT id, deleted_at FROM it_soft_users WHERE id = ?")
            .bind(u.id)
            .fetch_optional(pool)
            .await
            .expect("raw query after restore");

    let (_, deleted_at2) = row2.expect("row must still exist");
    assert!(
        deleted_at2.is_none(),
        "deleted_at must be NULL after restore"
    );

    // force_delete
    u.force_delete().await.expect("force delete");
    let gone: Option<(i32,)> = sqlx::query_as("SELECT id FROM it_soft_users WHERE id = ?")
        .bind(u.id)
        .fetch_optional(pool)
        .await
        .expect("raw query after force delete");
    assert!(gone.is_none(), "row must be gone after force_delete");

    Schema::drop_if_exists("it_soft_users")
        .await
        .expect("drop it_soft_users");
}

// ── Scenario 3: transactions ──────────────────────────────────────────────
#[derive(Debug, Clone, FromRow, rullst_orm::Orm)]
#[orm(table = "it_tx_accounts")]
struct Account {
    pub id: i32,
    pub balance: i32,
}

async fn scenario_transactions() {
    Schema::create("it_tx_accounts", |t: &mut Blueprint| {
        t.id();
        t.integer("balance").not_null();
    })
    .await
    .expect("create it_tx_accounts");

    // Successful transaction
    {
        let pool = Orm::pool();
        let mut tx = pool.begin().await.expect("begin tx");
        let mut acc = Account {
            id: 0,
            balance: 100,
        };
        acc.save_with_tx(&mut tx).await.expect("save with tx");
        acc.balance = 200;
        acc.save_with_tx(&mut tx).await.expect("update with tx");
        tx.commit().await.expect("commit");

        let committed = Account::find(acc.id).await.expect("find").expect("exists");
        assert_eq!(committed.balance, 200, "committed balance must be 200");
    }

    // Rolled-back transaction — changes must not persist
    {
        let initial_count = Account::query().count().await.expect("count");

        let pool = Orm::pool();
        let mut tx = pool.begin().await.expect("begin tx2");
        let mut ghost = Account {
            id: 0,
            balance: 999,
        };
        ghost.save_with_tx(&mut tx).await.expect("save ghost");
        // rollback instead of commit
        tx.rollback().await.expect("rollback");

        let after_rollback = Account::query()
            .count()
            .await
            .expect("count after rollback");
        assert_eq!(
            after_rollback, initial_count,
            "rollback must not persist the ghost account"
        );
    }

    Schema::drop_if_exists("it_tx_accounts")
        .await
        .expect("drop it_tx_accounts");
}

// ── Scenario 4: JSON column round-trip ───────────────────────────────────
async fn scenario_json_column() {
    Schema::create("it_json_records", |t: &mut Blueprint| {
        t.id();
        t.string("data").not_null();
    })
    .await
    .expect("create it_json_records");

    let mut rec = JsonRecord {
        id: 0,
        data: Json(Payload {
            value: "hello_world".into(),
        }),
    };
    rec.save().await.expect("save json record");

    let fetched = JsonRecord::find(rec.id)
        .await
        .expect("find")
        .expect("exists");
    assert_eq!(
        fetched.data.0.value, "hello_world",
        "JSON round-trip must preserve value"
    );

    // Verify to_json / from_json
    let json_str = rec.to_json();
    assert!(
        json_str.contains("hello_world"),
        "to_json must include field value"
    );

    let rehydrated = JsonRecord::from_json(&json_str).expect("from_json");
    assert_eq!(rehydrated.data.0.value, "hello_world");

    Schema::drop_if_exists("it_json_records")
        .await
        .expect("drop it_json_records");
}

// ── Scenario 5: bulk operations ───────────────────────────────────────────
#[derive(Debug, Clone, FromRow, rullst_orm::Orm)]
#[orm(table = "it_bulk_items")]
struct BulkItem {
    pub id: i32,
    pub label: String,
    pub score: i32,
}

async fn scenario_bulk_operations() {
    Schema::create("it_bulk_items", |t: &mut Blueprint| {
        t.id();
        t.string("label").not_null();
        t.integer("score").not_null();
    })
    .await
    .expect("create it_bulk_items");

    // Insert 20 records
    for i in 1..=20i32 {
        let mut item = BulkItem {
            id: 0,
            label: format!("item_{}", i),
            score: i,
        };
        item.save().await.expect("bulk save");
    }

    // ORDER BY + LIMIT
    let top5 = BulkItem::query()
        .order_by_desc("score")
        .limit(5)
        .get()
        .await
        .expect("top 5");
    assert_eq!(top5.len(), 5);
    assert_eq!(top5[0].score, 20, "highest score must be first");

    // OFFSET pagination
    let page2 = BulkItem::query()
        .order_by("score")
        .limit(5)
        .offset(5)
        .get()
        .await
        .expect("page 2");
    assert_eq!(page2.len(), 5);
    assert_eq!(page2[0].score, 6, "offset 5 → score 6");

    // pluck_i32
    let scores = BulkItem::query()
        .order_by("score")
        .limit(3)
        .pluck_i32("score")
        .await
        .expect("pluck scores");
    assert_eq!(scores, vec![1, 2, 3]);

    // pluck_string
    let labels = BulkItem::query()
        .order_by("score")
        .limit(2)
        .pluck_string("label")
        .await
        .expect("pluck labels");
    assert_eq!(labels, vec!["item_1", "item_2"]);

    // delete_all with WHERE
    let deleted = BulkItem::query()
        .where_eq("score", 1)
        .delete_all()
        .await
        .expect("delete score=1");
    assert_eq!(deleted, 1, "one row deleted");

    let count = BulkItem::query().count().await.expect("count after delete");
    assert_eq!(count, 19);

    Schema::drop_if_exists("it_bulk_items")
        .await
        .expect("drop it_bulk_items");
}

// ── Scenario 6: schema lifecycle ─────────────────────────────────────────
async fn scenario_schema_lifecycle() {
    // Create → verify → rename → drop_if_exists (idempotent)
    Schema::create("it_lifecycle_alpha", |t: &mut Blueprint| {
        t.id();
        t.string("value").not_null();
    })
    .await
    .expect("create it_lifecycle_alpha");

    // Inserting into the new table confirms it exists
    let pool = Orm::pool();
    sqlx::query("INSERT INTO it_lifecycle_alpha (value) VALUES (?)")
        .bind("check")
        .execute(pool)
        .await
        .expect("insert into lifecycle table");

    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM it_lifecycle_alpha")
        .fetch_one(pool)
        .await
        .expect("count lifecycle");
    assert_eq!(row.0, 1);

    // drop
    Schema::drop_if_exists("it_lifecycle_alpha")
        .await
        .expect("drop it_lifecycle_alpha");

    // Calling drop_if_exists on a non-existent table must not panic
    Schema::drop_if_exists("it_lifecycle_alpha")
        .await
        .expect("drop_if_exists on missing table must succeed");
}

// ── Scenario 7: audit logging ─────────────────────────────────────────────
async fn scenario_audit() {
    rullst_orm::audit::create_audit_table()
        .await
        .expect("create audit table");

    rullst_orm::audit::log_audit(
        "User",
        99,
        "created",
        None,
        Some(r#"{"name":"test"}"#.to_string()),
    )
    .await
    .expect("log audit");

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM rullst_audits")
        .fetch_one(Orm::pool())
        .await
        .expect("count audits");
    assert_eq!(count.0, 1);

    rullst_orm::schema::Schema::drop_if_exists("rullst_audits")
        .await
        .expect("drop audits");
}

// ── Scenario 8: query result ext ──────────────────────────────────────────
async fn scenario_query_result_ext() {
    use rullst_orm::database::QueryResultExt;

    Schema::create("it_query_result_ext", |t: &mut Blueprint| {
        t.id();
        t.string("name").not_null();
    })
    .await
    .expect("create it_query_result_ext");

    let pool = Orm::pool();
    let result = sqlx::query("INSERT INTO it_query_result_ext (name) VALUES ('Test')")
        .execute(pool)
        .await
        .expect("insert");

    #[cfg(not(any(
        feature = "strict-postgres",
        feature = "strict-mysql",
        feature = "strict-sqlite"
    )))]
    {
        let id = result.get_last_insert_id();
        assert!(id >= 0, "last insert id should be >= 0");
    }

    Schema::drop_if_exists("it_query_result_ext")
        .await
        .expect("drop it_query_result_ext");
}
