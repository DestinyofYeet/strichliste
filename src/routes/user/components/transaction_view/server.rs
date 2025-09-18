use leptos::prelude::*;

use crate::models::{Page, PageRequestParams, Transaction, UserId};

#[cfg(feature = "ssr")]
use {
    crate::{
        models::{Group, GroupId, Money, TransactionDB, User},
        routes::user::get_user,
    },
    tracing::{debug, error, trace, warn},
};

#[server]
pub async fn get_group_members(gid: i64) -> Result<Vec<String>, ServerFnError> {
    use crate::backend::core::ServerState;
    let state: ServerState = expect_context();
    use axum::http::StatusCode;
    use leptos_axum::ResponseOptions;
    let response_opts: ResponseOptions = expect_context();

    let db = state.db.lock().await;

    let mut conn = match db.get_conn().await {
        Ok(val) => val,
        Err(e) => {
            error!("Failed to get db handle: {e}");
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            return Err(ServerFnError::new("Failed to get database!"));
        }
    };

    let group = match Group::get(&mut *conn, GroupId(gid)).await {
        Ok(val) => val,
        Err(e) => {
            error!("Failed to get group from db: {e}");
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            return Err(ServerFnError::new("Failed to get group"));
        }
    };

    Ok(group
        .members
        .into_iter()
        .map(|elem| elem.nickname)
        .collect())
}

#[server]
pub async fn get_user_transactions(
    user_id: UserId,
    page_request_params: PageRequestParams,
) -> Result<Page<Transaction>, ServerFnError> {
    use crate::backend::core::ServerState;
    let state: ServerState = expect_context();
    use axum::http::StatusCode;
    use leptos_axum::ResponseOptions;
    let response_opts: ResponseOptions = expect_context();

    let transactions = match Transaction::get_user_transactions(
        &*state.db.lock().await,
        user_id,
        page_request_params,
    )
    .await
    {
        Ok(transactions) => transactions,
        Err(err) => {
            error!("Failed to fetch transactions: {}", err.to_string());
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            return Err(ServerFnError::new("Failed to fetch transactions!"));
        }
    };

    Ok(transactions)
}

#[server]
pub async fn undo_transaction(user_id: UserId, transaction_id: i64) -> Result<(), ServerFnError> {
    use crate::backend::core::ServerState;
    let state: ServerState = expect_context();
    use axum::http::StatusCode;
    use leptos_axum::ResponseOptions;

    let response_opts: ResponseOptions = expect_context();

    debug!(
        "Need to undo transaction {} for user {}",
        transaction_id, user_id
    );

    let user = get_user(user_id).await?;

    let user = match user {
        Some(user) => user,
        None => {
            warn!("A user with id '{}' does not exist!", user_id);
            response_opts.set_status(StatusCode::BAD_REQUEST);
            return Err(ServerFnError::new("Invalid user!"));
        }
    };

    let db = state.db.lock().await;

    let mut db_trns = match db.get_conn_transaction().await {
        Ok(value) => value,
        Err(e) => {
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            error!("Failed to get connection to db: {}", e);
            return Err(ServerFnError::new(
                "Failed to create connection to database!",
            ));
        }
    };

    let transaction_db = match TransactionDB::get(&mut *db_trns, transaction_id).await {
        Ok(value) => value,
        Err(e) => {
            error!("Failed to fetch transactions: {}", e);
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            return Err(ServerFnError::new("Failed to fetch transaction!"));
        }
    };

    let transaction_db = match transaction_db {
        None => {
            warn!("A transaction with id '{}' does not exist!", transaction_id);
            response_opts.set_status(StatusCode::BAD_REQUEST);
            return Err(ServerFnError::new("Invalid transaction!"));
        }
        Some(value) => value,
    };

    if transaction_db.is_undone {
        warn!("Attempting to undo a transaction that is already undone!");
        response_opts.set_status(StatusCode::BAD_REQUEST);
        return Err(ServerFnError::new("The transaction is already undone!"));
    }

    let (receiver_group, sender_group) = (
        match Group::get(&mut *db_trns, GroupId(transaction_db.receiver)).await {
            Ok(val) => val,
            Err(e) => {
                warn!(
                    "Failed to find receiver group with id '{}': {}",
                    transaction_db.receiver, e
                );
                response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
                return Err(ServerFnError::new("Failed to find group!"));
            }
        },
        match Group::get(&mut *db_trns, GroupId(transaction_db.sender)).await {
            Ok(val) => val,
            Err(e) => {
                warn!(
                    "Failed to find sender group with id '{}': {}",
                    transaction_db.sender, e
                );
                response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
                return Err(ServerFnError::new("Failed to find group!"));
            }
        },
    );

    let money_shares_sender = transaction_db.money / sender_group.members.len() as u64;

    let mut full_money = transaction_db.money;
    trace!("full_money: {}", full_money);

    let mut users_sender = Vec::<User>::new();
    let mut users_receiver = Vec::<User>::new();

    for user_db in sender_group.members.iter() {
        let user = match User::get(&mut *db_trns, UserId(user_db.id)).await {
            Ok(val) => val,
            Err(e) => {
                error!("Failed to lookup user: '{}'", e);
                response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
                return Err(ServerFnError::new("Failed to find user"));
            }
        };

        let user = match user {
            Some(val) => val,
            None => {
                error!("Failed to find user with id '{}'", user_db.id);
                response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
                return Err(ServerFnError::new("Failed to find user"));
            }
        };

        users_sender.push(user);
    }

    for user_db in receiver_group.members.iter() {
        let user = match User::get(&mut *db_trns, UserId(user_db.id)).await {
            Ok(val) => val,
            Err(e) => {
                error!("Failed to lookup user: '{}'", e);
                response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
                return Err(ServerFnError::new("Failed to find user"));
            }
        };

        let user = match user {
            Some(val) => val,
            None => {
                error!("Failed to find user with id '{}'", user_db.id);
                response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
                return Err(ServerFnError::new("Failed to find user"));
            }
        };

        users_receiver.push(user);
    }

    for user_sender in users_sender.iter_mut() {
        match user_sender
            .add_money(
                &mut *db_trns,
                Money {
                    value: money_shares_sender as i64,
                },
            )
            .await
        {
            Ok(val) => val,
            Err(e) => {
                error!(
                    "Failed to add money to user '{}': {}",
                    user_sender.nickname, e
                );
                response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
                return Err(ServerFnError::new("Failed to subtract money"));
            }
        }

        full_money -= money_shares_sender;
        trace!("full_money: {}", full_money);
    }

    while full_money > 0 {
        for user_sender in users_sender.iter_mut() {
            match user_sender
                .add_money(&mut *db_trns, Money { value: 1 })
                .await
            {
                Ok(val) => val,
                Err(e) => {
                    error!(
                        "Failed to add money to user (fine-grained) '{}': {}",
                        user.nickname, e
                    );
                    response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
                    return Err(ServerFnError::new("Failed to subtract money"));
                }
            };

            trace!("full_money: {}", full_money);
            if full_money == 0 {
                break;
            }

            full_money -= 1;
        }
    }

    let restore_money_share = transaction_db.money / users_receiver.len() as u64;

    for user_receiver in users_receiver.iter_mut() {
        match user_receiver
            .add_money(
                &mut *db_trns,
                Money {
                    value: -(restore_money_share as i64),
                },
            )
            .await
        {
            Ok(val) => val,
            Err(e) => {
                error!(
                    "Failed to remove money from user '{}': {}",
                    user_receiver.nickname, e
                );
                response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
                return Err(ServerFnError::new("Failed to subtract money"));
            }
        }

        full_money += restore_money_share;
    }

    while full_money < transaction_db.money {
        for user_receiver in users_receiver.iter_mut() {
            match user_receiver
                .add_money(&mut *db_trns, Money { value: -1 })
                .await
            {
                Ok(val) => val,
                Err(e) => {
                    error!(
                        "Failed to remove money from user (fine-grained) '{}': {}",
                        user.nickname, e
                    );
                    response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
                    return Err(ServerFnError::new("Failed to subtract money"));
                }
            };

            if full_money == transaction_db.money {
                break;
            }

            full_money += 1;
        }
    }

    match TransactionDB::set_undone(&mut *db_trns, transaction_db.id, true).await {
        Ok(_) => {}
        Err(e) => {
            error!("Failed to set transaction to undone: {}", e);
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            return Err(ServerFnError::new("Failed to update transaction!"));
        }
    }

    match db_trns.commit().await {
        Ok(_) => {}
        Err(e) => {
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            error!("Failed to apply transaction: {}", e);
            return Err(ServerFnError::new("Failed to apply transaction!"));
        }
    }

    Ok(())
}
