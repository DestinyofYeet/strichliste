use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use leptos::{
    leptos_dom::logging::console_log,
    prelude::{expect_context, Get, Memo, RwSignal, ServerFnError, Set},
    server::Resource,
};
use leptos_router::hooks::use_params_map;
use reactive_stores::{Store, StoreField};

use crate::{
    models::{AudioPlayback, User, UserId},
    routes::user::load_user,
};

#[derive(Clone, Debug, Default, Store)]
pub struct FrontendStore {
    // #[store(key: i64 = |user| user.id.unwrap())]
    pub cached_sounds: HashMap<AudioPlayback, String>,
    pub error: RwSignal<Option<String>>,
}

#[derive(Error, Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum GetUserError {
    #[error("no user present in url")]
    NoUserInURL,

    #[error("invalid user in url {0}")]
    InvalidUserInURL(String), //ParseIntError does not impl Deserialize

    #[error("error on user load: {0}")]
    UserLoad(ServerFnError),

    #[error("user with id {0} not present")]
    UserNotPresentError(UserId),
}

impl From<ServerFnError> for GetUserError {
    fn from(value: ServerFnError) -> Self {
        Self::UserLoad(value)
    }
}

//this will be moved to either routes or shared
pub fn get_user_id() -> Memo<Result<UserId, GetUserError>> {
    let params = use_params_map();
    Memo::new(move |_| -> Result<UserId, GetUserError> {
        let as_str = params.get().get("id").ok_or(GetUserError::NoUserInURL)?;
        as_str
            .parse::<i64>()
            .map(UserId)
            .map_err(|e| GetUserError::InvalidUserInURL(e.to_string()))
    })
}

pub fn get_user() -> Resource<Result<User, GetUserError>> {
    Resource::new(
        move || get_user_id().get(),
        async |user_id| match user_id {
            Ok(user_id) => Ok(load_user(user_id)
                .await?
                .ok_or(GetUserError::UserNotPresentError(user_id))?),
            Err(e) => Err(e),
        },
    )
}
pub fn set_error(msg: impl ToString) {
    let store = expect_context::<Store<FrontendStore>>();
    match store.error().writer() {
        Some(signal) => signal.set(Some(msg.to_string())),
        None => console_log("Failed to set Error"),
    };
}
