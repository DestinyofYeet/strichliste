#[cfg(feature = "ssr")]
use {
    crate::backend::{core::Settings, database::DB},
    std::sync::Arc,
    tokio::sync::Mutex,
};

pub type ServerState = Arc<State>;

#[cfg(feature = "ssr")]
pub struct State {
    pub db: Mutex<DB>,
    pub settings: Settings,
}
