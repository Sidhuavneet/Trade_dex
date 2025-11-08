// Application state module

use crate::services::clickhouse::ClickHouseService;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub clickhouse: Arc<ClickHouseService>,
}

