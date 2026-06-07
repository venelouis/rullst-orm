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
) -> Result<(), crate::Error> {
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

pub fn compute_diff(old_json: &str, new_json: &str) -> (Option<String>, Option<String>) {
    let old_val: serde_json::Value =
        serde_json::from_str(old_json).unwrap_or(serde_json::Value::Null);
    let new_val: serde_json::Value =
        serde_json::from_str(new_json).unwrap_or(serde_json::Value::Null);

    let mut diff_old = serde_json::Map::new();
    let mut diff_new = serde_json::Map::new();

    if let (Some(old_obj), Some(new_obj)) = (old_val.as_object(), new_val.as_object()) {
        for (k, v) in old_obj {
            if let Some(new_v) = new_obj.get(k) {
                if v != new_v {
                    diff_old.insert(k.clone(), v.clone());
                    diff_new.insert(k.clone(), new_v.clone());
                }
            }
        }
    }

    if diff_old.is_empty() && diff_new.is_empty() {
        return (None, None); // Nothing changed
    }

    let final_old = serde_json::to_string(&diff_old).ok();
    let final_new = serde_json::to_string(&diff_new).ok();

    (final_old, final_new)
}

pub async fn log_audit_diff(
    model_type: &str,
    model_id: i32,
    event: &str,
    old_json: &str,
    new_json: &str,
) -> Result<(), crate::Error> {
    let (final_old, final_new) = compute_diff(old_json, new_json);
    if final_old.is_none() && final_new.is_none() {
        return Ok(()); // Nothing changed
    }
    log_audit(model_type, model_id, event, final_old, final_new).await
}

pub async fn create_audit_table() -> Result<(), crate::Error> {
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

#[cfg(test)]
mod tests {
    use super::AuditLog;

    #[test]
    fn test_audit_log_serialization_round_trip() {
        let log = AuditLog {
            id: 1,
            model_type: "User".to_string(),
            model_id: 42,
            event: "created".to_string(),
            old_values: None,
            new_values: Some(r#"{"name":"Alice"}"#.to_string()),
            created_at: Some("2024-01-01T00:00:00Z".to_string()),
        };

        let json_str = serde_json::to_string(&log).expect("serialize");
        assert!(json_str.contains("\"model_type\":\"User\""));
        assert!(json_str.contains("\"event\":\"created\""));

        let deserialized: AuditLog = serde_json::from_str(&json_str).expect("deserialize");
        assert_eq!(deserialized.id, 1);
        assert_eq!(deserialized.model_id, 42);
        assert_eq!(deserialized.event, "created");
        assert!(deserialized.old_values.is_none());
    }

    #[test]
    fn test_audit_log_clone_debug() {
        let log = AuditLog {
            id: 5,
            model_type: "Post".to_string(),
            model_id: 99,
            event: "updated".to_string(),
            old_values: Some(r#"{"title":"Old"}"#.to_string()),
            new_values: Some(r#"{"title":"New"}"#.to_string()),
            created_at: None,
        };
        let cloned = log.clone();
        assert_eq!(cloned.model_type, "Post");
        // Debug must not panic
        let _ = format!("{:?}", cloned);
    }

    #[test]
    fn test_compute_diff_changes() {
        let old_json = r#"{"name":"Alice","age":30}"#;
        let new_json = r#"{"name":"Alice","age":31}"#;
        let (old_diff, new_diff) = super::compute_diff(old_json, new_json);
        assert_eq!(old_diff.unwrap(), r#"{"age":30}"#);
        assert_eq!(new_diff.unwrap(), r#"{"age":31}"#);
    }

    #[test]
    fn test_compute_diff_no_changes() {
        let json = r#"{"name":"Alice","age":30}"#;
        let (old_diff, new_diff) = super::compute_diff(json, json);
        assert!(old_diff.is_none());
        assert!(new_diff.is_none());
    }

    #[test]
    fn test_compute_diff_invalid_json() {
        let (old_diff, new_diff) = super::compute_diff("not json", "{invalid}");
        assert!(old_diff.is_none());
        assert!(new_diff.is_none());
    }
}
