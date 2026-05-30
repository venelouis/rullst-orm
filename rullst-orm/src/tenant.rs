use crate::RullstValue;
use std::future::Future;

tokio::task_local! {
    pub static CURRENT_TENANT: RullstValue;
}

pub async fn with_tenant<T, F, R>(tenant_id: T, f: F) -> R
where
    T: Into<RullstValue>,
    F: Future<Output = R>,
{
    CURRENT_TENANT.scope(tenant_id.into(), f).await
}

pub fn get_tenant_id() -> Option<RullstValue> {
    CURRENT_TENANT.try_with(|t| t.clone()).ok()
}
