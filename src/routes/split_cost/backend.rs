use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::routes::user::CreateTransactionError;

#[cfg(feature = "ssr")]
use {
    crate::backend::database::DBError,
    crate::models::{Group, GroupId, Money, Transaction, User, UserId},
    leptos_axum::redirect,
    tracing::error,
};

#[derive(Error, Debug, Clone, Deserialize, Serialize)]
pub enum SplitCostError {
    #[error("Server function error: {0}")]
    ServerFn(ServerFnErrorErr),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Failed to parse money: {0}")]
    MoneyParseError(String),

    #[error("Money error: {0}")]
    MoneyError(String),

    #[error("User with nickname '{0}' does not exist!")]
    UserDoesNotExistError(String),

    #[error("Failed to create transaction: {0}")]
    CreateTransactionError(String),

    #[error("{0} may not be empty")]
    MayNotBeEmptyError(String),
}

impl FromServerFnError for SplitCostError {
    type Encoder = server_fn::codec::JsonEncoding;

    fn from_server_fn_error(value: ServerFnErrorErr) -> Self {
        Self::ServerFn(value)
    }
}

impl From<CreateTransactionError> for SplitCostError {
    fn from(value: CreateTransactionError) -> Self {
        Self::CreateTransactionError(value.to_string())
    }
}

#[cfg(feature = "ssr")]
impl From<DBError> for SplitCostError {
    fn from(value: DBError) -> Self {
        Self::DatabaseError(value.to_string())
    }
}

#[server]
pub async fn split_cost(
    primary_user: String,
    secondary_users_input: Option<Vec<String>>,
    money: String,
    description: String,
) -> Result<(), SplitCostError> {
    use crate::backend::core::ServerState;
    let state: ServerState = expect_context();
    use axum::http::StatusCode;
    use leptos_axum::ResponseOptions;

    let response_opts: ResponseOptions = expect_context();

    let mut money: Money = match money.try_into() {
        Ok(value) => value,
        Err(e) => {
            response_opts.set_status(StatusCode::BAD_REQUEST);
            return Err(SplitCostError::MoneyParseError(e.to_string()));
        }
    };

    if primary_user.is_empty() {
        response_opts.set_status(StatusCode::BAD_REQUEST);
        return Err(SplitCostError::MayNotBeEmptyError("User".to_string()));
    }

    let secondary_users_input = match secondary_users_input {
        Some(val) => val,
        None => {
            response_opts.set_status(StatusCode::BAD_REQUEST);
            return Err(SplitCostError::MayNotBeEmptyError(
                "Other users".to_string(),
            ));
        }
    };

    if secondary_users_input.is_empty() {
        response_opts.set_status(StatusCode::BAD_REQUEST);
        return Err(SplitCostError::MayNotBeEmptyError(
            "Other users".to_string(),
        ));
    }

    let description = match description.is_empty() {
        true => None,
        false => Some(description),
    };

    if money.value <= 0 {
        response_opts.set_status(StatusCode::BAD_REQUEST);
        return Err(SplitCostError::MoneyError(
            "Money must be positive".to_string(),
        ));
    }

    let db = state.db.lock().await;
    let mut trans = match db.get_conn_transaction().await {
        Ok(val) => val,
        Err(e) => {
            error!("Failed to get database handle: {e}");
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            return Err(SplitCostError::DatabaseError(
                "Failed to get database handle!".to_string(),
            ));
        }
    };

    let primary_user = match User::get_by_nick(&mut *trans, &primary_user).await? {
        Some(val) => val,
        None => {
            response_opts.set_status(StatusCode::BAD_REQUEST);
            return Err(SplitCostError::UserDoesNotExistError(primary_user));
        }
    };

    let mut secondary_users: Vec<UserId> = Vec::new();

    for user_input in secondary_users_input.into_iter() {
        let user = match User::get_by_nick(&mut *trans, &user_input).await? {
            Some(val) => val.id,
            None => {
                response_opts.set_status(StatusCode::BAD_REQUEST);
                return Err(SplitCostError::UserDoesNotExistError(user_input));
            }
        };

        secondary_users.push(user);
    }

    let primary_group = Group::get_user_group_id(&mut *trans, primary_user.id).await?;

    let secondary_group =
        Group::get_group_id_for_multiple_users(&mut *trans, &secondary_users).await?;

    let total_amount_of_users = (secondary_users.len() + 1) as i64;

    let single_share = money.value / total_amount_of_users;

    // the primary user already payed his share
    money.value -= single_share;

    let settings = &state.settings;

    Transaction::create(
        &mut *trans,
        secondary_group,
        primary_group,
        crate::models::TransactionType::Sent(GroupId(0)),
        description,
        money,
        settings,
    )
    .await?;

    if let Err(e) = trans.commit().await {
        error!("Failed to commit transaction: {e}");
        response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
        return Err(SplitCostError::DatabaseError(
            "Failed to commit transaction".to_string(),
        ));
    }

    redirect("/");

    Ok(())
}
