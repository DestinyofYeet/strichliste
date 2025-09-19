use leptos::{ev, leptos_dom::logging::console_log, prelude::*, task::spawn_local};
use leptos_router::hooks::use_navigate;
#[cfg(feature = "ssr")]
use tracing::{debug, error};

use crate::models::{Money, User};

#[server]
pub async fn get_all_users() -> Result<Vec<User>, ServerFnError> {
    use crate::backend::core::ServerState;
    let state: ServerState = expect_context();
    use axum::http::StatusCode;
    use leptos_axum::ResponseOptions;

    let response_opts: ResponseOptions = expect_context();

    let users = match User::get_all(&*state.db.lock().await).await {
        Ok(users) => users,
        Err(err) => {
            let err = err.to_string();
            error!("Could not fetch users: {}", err);
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            return Err(ServerFnError::new(err));
        }
    };

    Ok(users)
}

#[server]
pub async fn get_user_by_barcode(barcode_string: String) -> Result<Option<User>, ServerFnError> {
    use crate::backend::core::ServerState;
    let state: ServerState = expect_context();
    use axum::http::StatusCode;
    use leptos_axum::ResponseOptions;

    let response_opts: ResponseOptions = expect_context();

    debug!("Attempting to fetch a user by barcode '{}'", barcode_string);

    if barcode_string.is_empty() {
        return Ok(None);
    }

    let user = match User::get_by_card_number(&*state.db.lock().await, barcode_string).await {
        Ok(user) => user,
        Err(err) => {
            let err = err.to_string();
            error!("Could not fetch user: {}", err);
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            return Err(ServerFnError::new("Failed to fetch user"));
        }
    };

    Ok(user)
}

#[component]
pub fn View() -> impl IntoView {
    view! {
        <div class="grid grid-cols-10 gap-10 py-10">
            <div class="col-span-1 pl-5 justify-self-center">
                <a href="/user/create" class="inline-block">
                    <div class="flex justify-center">
                        // joinked from: https://gist.github.com/ibelick/0c92c1aba54c2f7e8b3a7381426ed029
                        <button class="inline-flex h-10 w-10 items-center justify-center rounded-full bg-gray-50 text-black drop-shadow-sm transition-colors duration-150 hover:bg-gray-200">
                            "+"
                        </button>
                    </div>
                </a>
            </div>
            {InvisibleScanInput()}
            <div class="col-span-9 pr-7">{ShowUsers()}</div>
        </div>
    }
}

#[component]
pub fn InvisibleScanInput() -> impl IntoView {
    let input_signal = RwSignal::new(String::new());

    let handle = window_event_listener(ev::keypress, move |ev| match ev.key().as_str() {
        "Enter" => {
            let scan_input = input_signal.read_untracked().clone();
            input_signal.write_only().set(String::new());

            if scan_input.is_empty() {
                return;
            }

            spawn_local(async move {
                let user = match get_user_by_barcode(scan_input.clone()).await {
                    Ok(user) => user,
                    Err(err) => {
                        console_log(&format!("Failed to fetch user by barcode: {}", err));
                        return;
                    }
                };

                let user = match user {
                    Some(user) => user,
                    None => {
                        console_log(&format!("There is no user with barcode \"{scan_input}\""));
                        return;
                    }
                };

                let navigate = use_navigate();
                navigate(&format!("/user/{}", user.id), Default::default());
            });
        }

        _ => {
            let mut prev = input_signal.read_untracked().clone();
            prev.push_str(&ev.key());
            input_signal.write_only().set(prev);
        }
    });

    on_cleanup(move || {
        handle.remove();
    });
}

#[component]
pub fn ShowUsers() -> impl IntoView {
    // use reactive_stores::Store;
    // let store = expect_context::<Store<FrontendStore>>();

    // let fetch_users = RwSignal::new(0 as i64);

    let user_data = Resource::new(move || {}, |_| get_all_users());

    view! {
        <Suspense fallback=move || {
            view! { <h1>"Loading users..."</h1> }
        }>
            {move || {
                let users = match user_data.get() {
                    Some(users) => users,
                    None => {
                        return view! {
                            <p class="bg-red-400 text-white text-center">"Failed to fetch users"</p>
                        }
                            .into_any();
                    }
                };
                let users = match users {
                    Ok(users) => users,
                    Err(err) => {
                        let error = err.to_string();
                        return view! {
                            <p class="text-red-900">"Failed to fetch users: "{error}</p>
                        }
                            .into_any();
                    }
                };

                // store.cached_users().writer().unwrap().clear();

                // store.cached_users().writer().unwrap().append(&mut users.clone());

                view! {
                    <div class="grid">
                        // manual fix, idk why tailwind does not take grid-cols-[repeat(auto-fill, minmax(8rem, 1fr))]
                        <div
                            class="grid gap-5"
                            style="grid-template-columns: repeat(auto-fill, minmax(8rem, 1fr));"
                        >
                            {users
                                .into_iter()
                                .map(|user| {
                                    let id = user.id;

                                    view! {
                                        <a href=format!("/user/{}", id)>
                                            <UserPreview user />
                                        </a>
                                    }
                                })
                                .collect_view()
                                .into_any()}
                        </div>
                    </div>
                }
                    .into_any()
            }}
        </Suspense>
    }
}

#[component]
pub fn UserPreview(user: User) -> impl IntoView {
    view! {
        <div class="flex flex-col bg-[#2e3d4d] gap-2 rounded-[10px] py-2">
            <p class="text-center text-white">{user.nickname.clone()}</p>
            <p
                class="text-center"
                class=("text-red-500", move || { user.money.value < 0 })
                class=("text-green-500", move || { user.money.value >= 0 })
            >
                {Money::format_eur_diff_value(user.money.value)}
            </p>
        </div>
    }
}
