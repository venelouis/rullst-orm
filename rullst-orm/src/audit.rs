use crate::Orm;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: i32,
    pub model_type: String,
    pub model_id: i32,
    pub event: String,
    pub old_values: Option<String>,
    pub new_values: Option<String>,
    pub created_at: Option<String>,
}

pub async fn log_audit(
    model_type: &str,
    model_id: i32,
    event: &str,
    old_values: Option<String>,
    new_values: Option<String>,
) -> Result<(), sqlx::Error> {
    let pool = Orm::pool();
    let driver = Orm::driver();

    if driver == "postgres" {
        sqlx::query(
            "INSERT INTO rullst_audits (model_type, model_id, event, old_values, new_values) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(model_type)
        .bind(model_id)
        .bind(event)
        .bind(old_values)
        .bind(new_values)
        .execute(pool)
        .await?;
    } else {
        sqlx::query(
            "INSERT INTO rullst_audits (model_type, model_id, event, old_values, new_values) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(model_type)
        .bind(model_id)
        .bind(event)
        .bind(old_values)
        .bind(new_values)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn log_audit_diff(
    model_type: &str,
    model_id: i32,
    event: &str,
    old_json: &str,
    new_json: &str,
) -> Result<(), sqlx::Error> {
    let old_val: serde_json::Value = serde_json::from_str(old_json).unwrap_or(serde_json::Value::Null);
    let new_val: serde_json::Value = serde_json::from_str(new_json).unwrap_or(serde_json::Value::Null);

    let mut diff_old = serde_json::Map::new();
    let mut diff_new = serde_json::Map::new();

    if let (Some(old_obj), Some(new_obj)) = (old_val.as_object(), new_val.as_object()) {
        for (k, v) in old_obj {
            if let Some(new_v) = new_obj.get(k) && v != new_v {
                diff_old.insert(k.clone(), v.clone());
                diff_new.insert(k.clone(), new_v.clone());
            }
        }
    }

    if diff_old.is_empty() && diff_new.is_empty() {
        return Ok(()); // Nothing changed
    }

    let final_old = serde_json::to_string(&diff_old).ok();
    let final_new = serde_json::to_string(&diff_new).ok();

    log_audit(model_type, model_id, event, final_old, final_new).await
}

pub async fn create_audit_table() -> Result<(), sqlx::Error> {
    let pool = Orm::pool();
    let driver = Orm::driver();

    let query = if driver == "postgres" {
        r#"
        CREATE TABLE IF NOT EXISTS rullst_audits (
            id SERIAL PRIMARY KEY,
            model_type VARCHAR(255) NOT NULL,
            model_id INT NOT NULL,
            event VARCHAR(50) NOT NULL,
            old_values TEXT,
            new_values TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#
    } else if driver == "mysql" {
        r#"
        CREATE TABLE IF NOT EXISTS rullst_audits (
            id INT AUTO_INCREMENT PRIMARY KEY,
            model_type VARCHAR(255) NOT NULL,
            model_id INT NOT NULL,
            event VARCHAR(50) NOT NULL,
            old_values TEXT,
            new_values TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#
    } else {
        r#"
        CREATE TABLE IF NOT EXISTS rullst_audits (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            model_type TEXT NOT NULL,
            model_id INTEGER NOT NULL,
            event TEXT NOT NULL,
            old_values TEXT,
            new_values TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#
    };

    sqlx::query(query).execute(pool).await?;
    Ok(())
}
