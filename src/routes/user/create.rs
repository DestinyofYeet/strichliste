use leptos::prelude::*;

#[cfg(feature = "ssr")]
use {
    crate::models::User,
    tracing::{debug, error},
};

#[server]
pub async fn create_user(username: String) -> Result<(), ServerFnError> {
    use crate::backend::core::ServerState;
    let state: ServerState = expect_context();

    use axum::http::StatusCode;
    use leptos_axum::redirect;
    use leptos_axum::ResponseOptions;

    let response_opts: ResponseOptions = expect_context();
    debug!("Creating account!");

    if username.is_empty() {
        response_opts.set_status(StatusCode::BAD_REQUEST);
        return Err(ServerFnError::new("Name cannot be empty!"));
    }
    let username = username.trim().to_string();

    let user_id = match User::create(&*state.db.lock().await, username, None).await {
        Ok(value) => value,
        Err(e) => {
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            error!("Failed to add user: {}", e);
            return Err(ServerFnError::new(e));
        }
    };

    redirect(&format!("/user/{}", user_id));

    Ok(())
}

#[component]
pub fn Create() -> impl IntoView {
    let create_user_action = ServerAction::<CreateUser>::new();
    view! {
        <div class="flex h-screen bg-gray-900">
            <div class="w-full max-w-xs m-auto bg-indigo-100 rounded p-5">
                <ActionForm action=create_user_action>
                    <div>
                        <label class="block mb-2 text-indigo-500" for="username">
                            Nickname
                        </label>
                        <input
                            autocomplete="off"
                            class="w-full p-2 mb-6 text-indigo-700 border-b-2 border-indigo-500 outline-none focus:bg-gray-300"
                            type="text"
                            name="username"
                        />
                    </div>
                    <div>
                        <input
                            class="w-full bg-indigo-700 hover:bg-pink-700 text-white font-bold py-2 px-4 mb-6 rounded"
                            type="submit"
                            value="Create Account"
                        />
                    </div>
                </ActionForm>
                <div>
                    {move || match create_user_action.value().get() {
                        Some(Err(e)) => {
                            let msg = match e {
                                ServerFnError::ServerError(msg) => msg,
                                _ => e.to_string(),
                            };

                            view! { <p class="text-red-900">"Failed to create user: "{msg}</p> }
                                .into_any()
                        }
                        _ => ().into_any(),
                    }}
                </div>
            </div>
        </div>
    }
}
