use leptos::{html, leptos_dom::logging::console_log, prelude::*, task::spawn_local};
use leptos_router::hooks::use_params_map;

use crate::{
    models::UserId,
    routes::{home::get_all_users, user::get_user},
};

#[cfg(feature = "ssr")]
use {
    crate::models::{GroupDB, GroupId},
    crate::models::{Money, Transaction, User},
    tracing::error,
};

#[server]
pub async fn send_money(
    user_id: UserId,
    to_user: String,
    amount: String,
) -> Result<(), ServerFnError> {
    use crate::backend::core::ServerState;
    let state: ServerState = expect_context();

    use axum::http::StatusCode;
    use leptos_axum::redirect;
    use leptos_axum::ResponseOptions;

    let response_opts: ResponseOptions = expect_context();

    let money: Money = match amount.clone().try_into() {
        Ok(value) => value,
        Err(e) => {
            response_opts.set_status(StatusCode::BAD_REQUEST);
            return Err(ServerFnError::new(format!(
                "Failed to convert '{amount}' to internal representation: {e}"
            )));
        }
    };

    if money.value < 0 {
        response_opts.set_status(StatusCode::BAD_REQUEST);
        return Err(ServerFnError::new("Amount to be sent must be > 0!"));
    }

    let sender = match get_user(user_id).await? {
        Some(value) => value,
        None => {
            response_opts.set_status(StatusCode::BAD_REQUEST);
            return Err(ServerFnError::new(
                "The user you are trying to send the money from does not exist!",
            ));
        }
    };

    let db = state.db.lock().await;

    let mut db_trns = match db.get_conn_transaction().await {
        Ok(value) => value,
        Err(e) => {
            error!("Failed to get db transaction: {}", e);
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            return Err(ServerFnError::new("Failed to acquire db transaction!"));
        }
    };

    let recipient = match User::get_by_nick(&mut *db_trns, &to_user.clone()).await {
        Ok(val) => val,
        Err(e) => {
            error!("Failed to lookup db: {e}");
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            return Err(ServerFnError::new("Failed to lookup db"));
        }
    };

    let recipient = match recipient {
        Some(val) => val,
        None => {
            response_opts.set_status(StatusCode::BAD_REQUEST);
            return Err(ServerFnError::new(format!(
                "Recipient '{to_user}' was not found!"
            )));
        }
    };

    if sender.id == recipient.id {
        response_opts.set_status(StatusCode::BAD_REQUEST);
        return Err(ServerFnError::new(
            "Sending and receiving user must not be the same!",
        ));
    }

    let sender_group = match GroupDB::get_single_group(&mut *db_trns, sender.id).await {
        Ok(val) => val,
        Err(e) => {
            error!("Failed to find single group: {e}");
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            return Err(ServerFnError::new("Failed to find single group"));
        }
    };

    let recipient_group = match GroupDB::get_single_group(&mut *db_trns, recipient.id).await {
        Ok(val) => val,
        Err(e) => {
            error!("Failed to find single group: {e}");
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            return Err(ServerFnError::new("Failed to find single group"));
        }
    };

    Transaction::create(
        &mut *db_trns,
        GroupId(sender_group),
        GroupId(recipient_group),
        crate::models::TransactionType::Sent(GroupId(recipient_group)),
        None,
        money,
        &state.settings,
    )
    .await?;

    if let Err(e) = db_trns.commit().await {
        error!("Failed to commit transaction: {}", e);
        response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
        return Err(ServerFnError::new("Failed to apply transaction!"));
    };

    redirect(&format!("/user/{}", sender.id));

    Ok(())
}

#[component]
pub fn Show() -> impl IntoView {
    let params = use_params_map();
    let user_id_string = params.read_untracked().get("id").unwrap_or_default();

    let user_id = match user_id_string.parse::<i64>() {
        Ok(value) => value,
        Err(_e) => {
            return view! { <p class="text-red-500">"Failed to convert id to a number!"</p> }
                .into_any();
        }
    };

    let user_id = UserId(user_id);

    let user_resource = OnceResource::new(get_user(user_id));
    let all_users_resource = OnceResource::new(get_all_users());

    let receiver_input = RwSignal::new(String::new());
    let amount_input = RwSignal::new(String::new());
    let hidden_div_node_ref = NodeRef::<html::Div>::new();

    let error_result = RwSignal::new(String::new());
    view! {
        <Suspense fallback=move || {
            view! { <p class="text-white text-center">"Loading user"</p> }
        }>
            {move || {
                user_resource
                    .get()
                    .map(|user| {
                        let user = match user {
                            Ok(value) => value,
                            Err(e) => {
                                let e = e.to_string();
                                return view! {
                                    <p class="bg-red-400 text-white text-center">
                                        "Failed to fetch user: "{e}
                                    </p>
                                }
                                    .into_any();
                            }
                        };
                        let user = match user {
                            Some(value) => value,
                            None => {
                                return view! {
                                    <p class="bg-red-400 text-white text-center">
                                        "No such user with id '"{user_id.0}"' exists!"
                                    </p>
                                }
                                    .into_any();
                            }
                        };
                        let all_users = match all_users_resource.get() {
                            Some(Ok(value)) => value,
                            _ => {
                                return view! {
                                    <p class="bg-red-400 text-white text-center">
                                        "Failed to fetch all users!"
                                    </p>
                                }
                                    .into_any();
                            }
                        };

                        view! {
                            <div class="flex h-screen bg-gray-900">
                                <div class="w-full max-w-xs m-auto bg-indigo-100 rounded p-5">
                                    <p class="text-indigo-500 text-center">
                                        "Hello "{user.nickname}", who do you want to send money to?"
                                    </p>
                                    <div>
                                        <label class="block mb-2 text-indigo-500">"Receiver"</label>
                                        <input
                                            bind:value=receiver_input
                                            autocomplete="off"
                                            class="w-full p-2 mb-6 text-indigo-700 border-b-2 border-indigo-500 outline-none focus:bg-gray-300"
                                            type="text"
                                            name="to_user"
                                        />
                                    </div>
                                    <div
                                        class="flex flex-col items-center"
                                        class=("hidden", move || receiver_input.get().is_empty())
                                        node_ref=hidden_div_node_ref
                                    >
                                        {move || {
                                            let search = receiver_input.get();
                                            all_users
                                                .iter()
                                                .filter(|elem| elem.id != user.id)
                                                .filter(|elem| {
                                                    elem.nickname
                                                        .to_lowercase()
                                                        .contains(&search.to_lowercase())
                                                })
                                                .map(|elem| {
                                                    let nickname = elem.nickname.clone();
                                                    let n_clone = nickname.clone();

                                                    view! {
                                                        <button
                                                            class="bg-gray-400 text-white p-2 rounded"
                                                            on:click=move |_| {
                                                                receiver_input.set(n_clone.clone());
                                                                hidden_div_node_ref
                                                                    .get()
                                                                    .map(|elem| {
                                                                        elem.class("hidden flex flex-col items-center")
                                                                    });
                                                            }
                                                        >
                                                            {nickname}
                                                        </button>
                                                    }
                                                })
                                                .collect_view()
                                        }}
                                    </div>
                                    <div>
                                        <label class="block mb-2 text-indigo-500">"Amount"</label>
                                        <input
                                            bind:value=amount_input
                                            autocomplete="off"
                                            class="w-full p-2 mb-6 text-indigo-700 border-b-2 border-indigo-500 outline-none focus:bg-gray-300"
                                            type="text"
                                            name="amount"
                                        />
                                    </div>
                                    <div>
                                        <input
                                            class="w-full bg-indigo-700 hover:bg-pink-700 text-white font-bold py-2 px-4 mb-6 rounded"
                                            type="submit"
                                            value="Send money"
                                            on:click=move |_| {
                                                spawn_local(async move {
                                                    console_log("Need to send_money ()");
                                                    match send_money(
                                                            user_id,
                                                            receiver_input.get(),
                                                            amount_input.get(),
                                                        )
                                                        .await
                                                    {
                                                        Ok(_) => {}
                                                        Err(e) => {
                                                            error_result.set(e.to_string());
                                                        }
                                                    }
                                                });
                                            }
                                        />
                                    </div>
                                    <div>
                                        {move || match error_result.get().len() {
                                            0 => ().into_any(),
                                            _ => {
                                                let msg = error_result.get();
                                                view! {
                                                    <p class="text-red-900">"Failed to send money:  "{msg}</p>
                                                }
                                                    .into_any()
                                            }
                                        }}
                                    </div>
                                </div>
                            </div>
                        }
                            .into_any()
                    })
            }}
        </Suspense>
    }.into_any()
}
