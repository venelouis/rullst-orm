#![cfg(not(any(feature = "strict-postgres", feature = "strict-mysql")))]

use rullst_orm::schema::{Blueprint, Schema};
use rullst_orm::types::Json;
use rullst_orm::{FromRow, Orm};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct MyJsonData {
    field: String,
}

#[derive(Debug, Clone, FromRow, rullst_orm::Orm)]
#[orm(table = "test_records")]
struct TestRecord {
    pub id: i32,
    pub name: String,
    pub data: Json<MyJsonData>,
}

#[tokio::test]
async fn test_schema_and_transaction() {
    // 1. Initialize DB
    // We use a file-based SQLite database for tests because sqlx::Any driver
    // requires a persistent connection for in-memory DBs to persist across queries.
    let db_path = "integration_test_rwc.db";
    let _ = std::fs::remove_file(db_path);
    Orm::init(&format!("sqlite:{}?mode=rwc", db_path))
        .await
        .unwrap();

    // 2. Test Schema::create
    let create_result = Schema::create("test_records", |table: &mut Blueprint| {
        table.id();
        table.string("name").not_null();
        table.string("data").not_null();
    })
    .await;
    assert!(create_result.is_ok(), "Schema::create failed");

    // 3. Test db::Transaction and JSON Serialization/Deserialization
    let pool = Orm::pool();
    let mut tx: rullst_orm::db::Transaction = pool.begin().await.unwrap();

    let mut record = TestRecord {
        id: 0,
        name: "Record1".to_string(),
        data: Json(MyJsonData {
            field: "value1".to_string(),
        }),
    };

    // Save with transaction
    record.save_with_tx(&mut tx).await.unwrap();

    // Verify it within transaction
    let fetched = TestRecord::query()
        .where_eq("name", "Record1")
        .first_with_tx(&mut tx)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fetched.name, "Record1");
    assert_eq!(fetched.data.0.field, "value1"); // Verifies JsonDecode

    tx.commit().await.unwrap();

    // Verify after commit
    let fetched_after = TestRecord::find(record.id).await.unwrap().unwrap();
    assert_eq!(fetched_after.data.0.field, "value1");

    // 4. Test Schema::drop_if_exists
    let drop_result = Schema::drop_if_exists("test_records").await;
    assert!(drop_result.is_ok(), "Schema::drop_if_exists failed");

    // Table is dropped, querying should fail
    let query_after_drop = TestRecord::all().await;
    assert!(
        query_after_drop.is_err(),
        "Query should fail after table is dropped"
    );
}
