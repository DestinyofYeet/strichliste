use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::{models::UserId, routes::user::get_user};

#[cfg(feature = "ssr")]
use {
    crate::models::User,
    tracing::{debug, error, warn},
};

#[server]
pub async fn update_user(
    id: UserId,
    nickname: String,
    card_number: String,
) -> Result<(), ServerFnError> {
    use crate::backend::core::ServerState;
    let state: ServerState = expect_context();
    use axum::http::StatusCode;
    use leptos_axum::redirect;
    use leptos_axum::ResponseOptions;

    let response_opts: ResponseOptions = expect_context();

    let user = match get_user(id).await {
        Ok(user) => user,
        Err(err) => {
            error!("Failed to fetch user: {}", err.to_string());
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            return Err(ServerFnError::new("Failed to fetch user!"));
        }
    };

    let mut user = match user {
        Some(user) => user,
        None => {
            warn!("No such user with id '{}' exists!", id);
            response_opts.set_status(StatusCode::BAD_REQUEST);
            return Err(ServerFnError::new("No such user exists!"));
        }
    };

    match User::get_by_card_number(&*state.db.lock().await, card_number.clone()).await {
        Ok(value) => match value {
            None => {}
            Some(user) => {
                if user.id != id {
                    warn!("The card number '{}' is already used!", card_number);
                    response_opts.set_status(StatusCode::BAD_REQUEST);
                    return Err(ServerFnError::new("The card number is already used!"));
                }
            }
        },

        Err(e) => {
            error!("Failed to check for existence of the card number: {}", e);
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            return Err(ServerFnError::new(
                "Failed to check if the card number is already used!",
            ));
        }
    }

    let card_number = match card_number.len() {
        0 => None,
        _ => Some(card_number),
    };

    let db = &*state.db.lock().await;

    let mut db_trans = match db.get_conn_transaction().await {
        Ok(value) => value,
        Err(e) => {
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            error!("Failed to get database handle: {}", e);
            return Err(ServerFnError::new("Faile to get a database handle!"));
        }
    };

    match user.set_name(&mut *db_trans, nickname).await {
        Ok(_) => {}
        Err(e) => {
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            error!("Failed to set a new username: {}", e);
            return Err(ServerFnError::new("Failed to set a new username!"));
        }
    }

    debug!(
        "Changing card number for user '{}' to '{:?}'",
        user.id, user.card_number
    );

    match user.set_card_number(&mut *db_trans, card_number).await {
        Ok(_) => {}
        Err(e) => {
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            error!("Failed to set a new card number: {}", e);
            return Err(ServerFnError::new("Failed to set a new card number!"));
        }
    }

    match db_trans.commit().await {
        Ok(_) => {}
        Err(e) => {
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            error!("Failed to commit database transaction: {}", e);
            return Err(ServerFnError::new(
                "Failed to commit the database transaction",
            ));
        }
    }

    redirect(&format!("/user/{}", id));

    Ok(())
}

#[component]
pub fn Show() -> impl IntoView {
    let params = use_params_map();
    let user_id_string = params.read_untracked().get("id").unwrap_or_default();

    let user_id = match user_id_string.parse::<i64>() {
        Ok(user_id) => UserId(user_id),
        Err(_) => {
            return view! { <p class="text-red-500">"Failed to convert id to a number!"</p> }
                .into_any();
        }
    };

    let user_resource = OnceResource::new(get_user(user_id));

    let update_action = ServerAction::<UpdateUser>::new();
    view! {
        <Suspense fallback=move || {
            view! { <p class="text-white text-center pt-5">"Loading User..."</p> }
        }>
            {move || {
                let user = match user_resource.get() {
                    Some(user) => user,
                    None => {
                        return view! { <p class="text-red-500">"Failed to fetch user"</p> }
                            .into_any();
                    }
                };
                let user = match user {
                    Ok(user) => user,
                    Err(err) => {
                        let err = err.to_string();
                        return view! {
                            <p class="text-red-500">"Failed to fetch user because: "{err}</p>
                        }
                            .into_any();
                    }
                };
                let user = match user {
                    Some(user) => user,
                    None => {
                        return view! {
                            <p class="text-red-500">
                                "No user with the id "{user_id.0}" has been found!"
                            </p>
                        }
                            .into_any();
                    }
                };

                view! {
                    {move || match update_action.value().get() {
                        Some(Err(e)) => {
                            let msg = match e {
                                ServerFnError::ServerError(msg) => msg,
                                _ => e.to_string(),
                            };

                            view! {
                                <p class="p-3 bg-red-400 text-white text-center">
                                    "Failed to update user: "{msg}
                                </p>
                            }
                                .into_any()
                        }
                        _ => ().into_any(),
                    }}
                    <ActionForm action=update_action>
                        <div class="flex flex-col items-center gap-5">
                            <div class="flex flex-col items-center">
                                <label class="text-white text-[1.25em]">"Nickname"</label>
                                <input
                                    class="text-[1.25em]"
                                    type="text"
                                    value=user.nickname
                                    name="nickname"
                                />
                            </div>
                            <div class="flex flex-col items-center">
                                <label class="text-white text-[1.25em]">"Card number"</label>
                                <input
                                    class="text-[1.25em]"
                                    type="text"
                                    value=user.card_number
                                    name="card_number"
                                />
                            </div>
                            <input type="hidden" value=user.id.0 name="id" />
                            <input
                                class="text-white hover:bg-pink-700 bg-emerald-700 rounded-full text-[1.25em] p-2"
                                type="submit"
                                value="Update user"
                            />
                        </div>
                    </ActionForm>
                }
                    .into_any()
            }}
        </Suspense>
    }
    .into_any()
}
