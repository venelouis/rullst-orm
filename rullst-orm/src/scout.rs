use async_trait::async_trait;
use serde_json::Value;
use std::sync::OnceLock;

#[async_trait]
pub trait SearchEngine: Send + Sync {
    async fn update(&self, table: &str, id: i32, payload: Value) -> Result<(), sqlx::Error>;
    async fn delete(&self, table: &str, id: i32) -> Result<(), sqlx::Error>;
    async fn search(&self, table: &str, query: &str) -> Result<Vec<i32>, sqlx::Error>;
}

static SEARCH_ENGINE: OnceLock<Box<dyn SearchEngine>> = OnceLock::new();

pub fn set_search_engine(engine: Box<dyn SearchEngine>) {
    let _ = SEARCH_ENGINE.set(engine);
}

pub fn get_search_engine() -> Option<&'static dyn SearchEngine> {
    SEARCH_ENGINE.get().map(|e| &**e)
}
