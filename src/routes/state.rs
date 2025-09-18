use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use leptos::{
    leptos_dom::logging::console_log,
    prelude::{expect_context, Get, Memo, Read, RwSignal, ServerFnError, Set},
    server::Resource,
};
use leptos_router::params::ParamsMap;
use reactive_stores::{Store, StoreField};

use crate::{
    models::{AudioPlayback, User, UserId},
    routes::user::load_user,
};

#[derive(Clone, Debug, Store)]
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
pub fn get_user_id(params: Memo<ParamsMap>) -> Memo<Result<UserId, GetUserError>> {
    console_log(&format!("re-rendering get_user_id(): {:#?}", params));
    Memo::new(move |_| {
        match params.read().get("id").map(|id| {
            id.parse::<i64>()
                .map(UserId)
                .map_err(|e| GetUserError::InvalidUserInURL(e.to_string()))
        }) {
            Some(value) => value,
            None => Err(GetUserError::NoUserInURL),
        }
    })
}

pub fn get_user(
    user_id: Memo<Result<UserId, GetUserError>>,
) -> Resource<Result<User, GetUserError>> {
    Resource::new(
        move || user_id.get(),
        async |user_id| {
            console_log(&format!("executing resource. user_id: {:#?}", user_id));
            match user_id {
                Ok(user_id) => Ok(load_user(user_id)
                    .await?
                    .ok_or(GetUserError::UserNotPresentError(user_id))?),
                Err(e) => Err(e),
            }
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
