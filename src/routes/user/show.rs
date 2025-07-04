use std::rc::Rc;

use leptos::{prelude::*, task::spawn_local};
use leptos_router::hooks::use_params_map;
use tracing::error;

use crate::{
     models::{Money, Transaction,  TransactionType, User}, routes::user::components::{buy_article::BuyArticle, scan_input::invisible_scan_input}}
;
#[cfg(feature = "ssr")]
use crate::backend::db::{DBUSER_AUFLADUNG_ID, DBUSER_KASSE_ID};

use super::components::transaction_view::ShowTransactions;

#[derive(Debug, Clone)]
pub struct MoneyArgs {
    pub user_id: i64,
    pub money: RwSignal<Money>,
    pub error: RwSignal<String>,
    pub transactions: RwSignal<Vec<Transaction>>,
}

#[server]
pub async fn get_user(id: i64) -> Result<Option<User>, ServerFnError> {
    use crate::backend::ServerState;
    let state: ServerState = expect_context();
    use axum::http::StatusCode;
    use leptos_axum::ResponseOptions;

    let response_opts: ResponseOptions = expect_context();

    let db = state.db.lock().await;
    let mut conn = match db.get_conn().await {
        Ok(value) => value,
        Err(e) => {
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            error!("Failed to create database transaction: {}", e);
            return Err(ServerFnError::new("Failed to create database transaction"));
        }
    };

    let user = match User::get(&mut *conn, id).await {
        Ok(value) => value,
        Err(e) => {            
            error!("Failed to fetch user: {}", e);
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            return Err(ServerFnError::new(e));
        }
    };

    Ok(user)
}

#[server]
pub async fn create_transaction(user_id: i64, money: Money, transaction_type: TransactionType) -> Result<Transaction, ServerFnError> {
    use crate::backend::ServerState;
    use axum::http::StatusCode;
    use leptos_axum::ResponseOptions;

    let state: ServerState = expect_context();

    let response_opts: ResponseOptions = expect_context();

    let mut user = match get_user(user_id).await? {
        None => {
            response_opts.set_status(StatusCode::BAD_REQUEST);
            return Err(ServerFnError::new(&format!(
                "No user found with id {}",
                user_id
            )));
        },

        Some(value) => value
    };

    let db = state.db.lock().await;
    let mut db_trans = match db.get_conn_transaction().await {
        Ok(value) => value,
        Err(e) => {
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            error!("Failed to get database handle: {}", e);
            return Err(ServerFnError::new("Failed to get database handle!"));
        }
    };

    let (sender_id, receiver_id) = match transaction_type {
        TransactionType::DEPOSIT => (DBUSER_AUFLADUNG_ID, user_id),
        TransactionType::WITHDRAW => (user_id, DBUSER_AUFLADUNG_ID),
        TransactionType::BOUGHT(_) => (user_id, DBUSER_KASSE_ID),
        TransactionType::RECEIVED(tx_user) => (tx_user, user_id),
        TransactionType::SENT(rx_user) => (user_id, rx_user),
    };

    let transaction_id = match Transaction::create(&mut *db_trans, sender_id, receiver_id,  transaction_type, None, money).await {
        Ok(value) => value,
        Err(e) => {
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            error!("Failed to create transaction: {}", e);
            return Err(ServerFnError::new("Failed to create transaction!"));
        }
    };

    let transaction = match Transaction::get(&mut *db_trans, transaction_id, user_id).await {
        Ok(Some(value)) => value,
        _ => {
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            error!("Failed to find newly created transaction");
            return Err(ServerFnError::new("Failed to find newly created transaction!"));
        }
    };

    let new_value = user.money.value + transaction.money.value;

    match user.set_money(&mut *db_trans, new_value).await {
        Ok(_) => {},
        Err(e) => {
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            error!("Failed to update user: {}", e);
            return Err(ServerFnError::new("Failed to update user!"));
        }
    }

    match db_trans.commit().await {
        Ok(_) => {},
        Err(e) => {
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            error!("Failed to commit transaction: {}", e);
            return Err(ServerFnError::new("Failed to commit transaction!"));
        }
    }

    Ok(transaction)
}

#[component]
pub fn ShowUser() -> impl IntoView {
    let params = use_params_map();
    let user_id_string = params.read_untracked().get("id").unwrap_or_default();

    let user_id = user_id_string.parse::<i64>();

    if user_id.is_err() {
        return view! {
            <p class="text-red-500">"Failed to convert id to a number!"</p>
        }
        .into_any();
    }

    let user_id = user_id.unwrap();

    let user_resource = OnceResource::new(get_user(user_id));

    let error_signal = RwSignal::new(String::new());

    return view! {
        {
            move || {
                let error = error_signal.get();

                if error.len() != 0 {
                    view! {
                        
                        <div>
                            <p class="text-white bg-red-400 p-5 text-center">"An error has occurred: "{error}</p>
                        </div>
                    }.into_any()
                } else {
                    view! {}.into_any()
                }

            }
        }
        { move || {

            view!{
                <Suspense
                    fallback=move || view!{<p class="text-white text-center pt-5">"Loading user..."</p>}
                >
                <div>
                    {
                        move || {
                            let user = user_resource.get();

                            if user.is_none() {
                                return view!{
                                    <p class="text-red-500">"Failed to fetch user"</p>
                                }.into_any();
                            }

                            let user = user.unwrap();

                            if user.is_err(){
                                let err = user.err().unwrap().to_string();
                                return view!{
                                    <p class="text-red-500">"Failed to fetch user because: "{err}</p>
                                }.into_any();
                            }

                            let user = user.unwrap();

                            if user.is_none(){
                                return view! {
                                    <p class="text-red-500">"No user with the id "{user_id}" has been found!"</p>
                                }.into_any();
                            }

                            let user = user.unwrap();

                            let money_signal = RwSignal::new(user.money);

                            let transactions = RwSignal::new(Vec::<Transaction>::new());

                            let m_args = MoneyArgs {
                                user_id: user_id,
                                money: money_signal,
                                error: error_signal,
                                transactions: transactions,
                            };

                            let args1 = m_args.clone();
                            let args2 = m_args.clone();
                            
                            let args = Rc::new(m_args);

                            let custom_money_change = RwSignal::new(String::new());

                            let custom_money_is_focused = RwSignal::new(false);

                            view!{
                                {invisible_scan_input(custom_money_is_focused, error_signal, args.clone(), user_id)}
                                <div class="grid grid-cols-2">
                                    <div class="pt-5">
                                        // left side (show user statistics)
                                        <div class="grid grid-cols-3">
                                            <div class="col-span-2">
                                                <p class="text-center text-white text-[2em]">{user.nickname.clone()}</p>
                                                <p class="text-center text-[2em]"
                                                    class=("text-red-500", move || (money_signal.get()).value < 0)
                                                    class=("text-green-500", move || (money_signal.get()).value >= 0)

                                                >{move || (money_signal.get()).format_eur_diff()}</p>
                                                <div class="flex place-content-evenly">
                                                </div>
                                            </div>
                                            <div class="col-span-1">
                                                <div class="flex justify-evenly">
                                                    <a href=format!("/user/{}/settings", user_id) class="text-white pt-[5px] flex flex-col items-center">
                                                        {SettingsIcon()}
                                                        <p class="text-center">"Settings"</p>
                                                    </a>
                                                    <a href=format!("/user/{}/send_money", user_id) class="text-white w-[3rem] flex flex-col items-center">
                                                        <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><g id="SVGRepo_bgCarrier" stroke-width="0"></g><g id="SVGRepo_tracerCarrier" stroke-linecap="round" stroke-linejoin="round"></g><g id="SVGRepo_iconCarrier"> <path d="M12 3V9M12 3L9.5 5.5M12 3L14.5 5.5M5.82333 9.00037C6.2383 9.36683 6.5 9.90285 6.5 10.5C6.5 11.6046 5.60457 12.5 4.5 12.5C3.90285 12.5 3.36683 12.2383 3.00037 11.8233M5.82333 9.00037C5.94144 9 6.06676 9 6.2 9H8M5.82333 9.00037C4.94852 9.00308 4.46895 9.02593 4.09202 9.21799C3.71569 9.40973 3.40973 9.71569 3.21799 10.092C3.02593 10.469 3.00308 10.9485 3.00037 11.8233M3.00037 11.8233C3 11.9414 3 12.0668 3 12.2V17.8C3 17.9332 3 18.0586 3.00037 18.1767M3.00037 18.1767C3.36683 17.7617 3.90285 17.5 4.5 17.5C5.60457 17.5 6.5 18.3954 6.5 19.5C6.5 20.0971 6.2383 20.6332 5.82333 20.9996M3.00037 18.1767C3.00308 19.0515 3.02593 19.5311 3.21799 19.908C3.40973 20.2843 3.71569 20.5903 4.09202 20.782C4.46895 20.9741 4.94852 20.9969 5.82333 20.9996M5.82333 20.9996C5.94144 21 6.06676 21 6.2 21H17.8C17.9332 21 18.0586 21 18.1767 20.9996M21 18.1771C20.6335 17.7619 20.0973 17.5 19.5 17.5C18.3954 17.5 17.5 18.3954 17.5 19.5C17.5 20.0971 17.7617 20.6332 18.1767 20.9996M21 18.1771C21.0004 18.0589 21 17.9334 21 17.8V12.2C21 12.0668 21 11.9414 20.9996 11.8233M21 18.1771C20.9973 19.0516 20.974 19.5311 20.782 19.908C20.5903 20.2843 20.2843 20.5903 19.908 20.782C19.5311 20.9741 19.0515 20.9969 18.1767 20.9996M20.9996 11.8233C20.6332 12.2383 20.0971 12.5 19.5 12.5C18.3954 12.5 17.5 11.6046 17.5 10.5C17.5 9.90285 17.7617 9.36683 18.1767 9.00037M20.9996 11.8233C20.9969 10.9485 20.9741 10.469 20.782 10.092C20.5903 9.71569 20.2843 9.40973 19.908 9.21799C19.5311 9.02593 19.0515 9.00308 18.1767 9.00037M18.1767 9.00037C18.0586 9 17.9332 9 17.8 9H16M14 15C14 16.1046 13.1046 17 12 17C10.8954 17 10 16.1046 10 15C10 13.8954 10.8954 13 12 13C13.1046 13 14 13.8954 14 15Z" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"></path> </g></svg>
                                                        <p class="text-center">"Send money"</p>
                                                    </a>
                                                </div>
                                            </div>
                                        </div>
                                        <BuyArticle args=args.clone()/>
                                    </div>
                                    <div>
                                        // right side (put in money)
                                        <div class="flex flex-col gap-3 bg-gray-500 p-3 rounded-[10px]">
                                            <div class="grid grid-cols-3 gap-5 rounded-[10px]">
                                                {change_money_button(50, args.clone())}
                                                {change_money_button(100, args.clone())}
                                                {change_money_button(200, args.clone())}
                                                {change_money_button(500, args.clone())}
                                                {change_money_button(1000, args.clone())}
                                                {change_money_button(2000, args.clone())}
                                                {change_money_button(5000, args.clone())}

                                            </div>
                                            <div class="grid grid-cols-3 gap-3">
                                                <a href="#" class="bg-red-400 text-white rounded-full p-5" on:click=move |_| on_custom_money_button_click(false, custom_money_change, &args1)>
                                                    <div class="pad-5 text-center">
                                                        "-"
                                                    </div>
                                                </a>
                                                <input class="text-center rounded-[10px]" placeholder="Euros" bind:value=custom_money_change value="00.00"
                                                    on:focus=move |_| {custom_money_is_focused.set(true)}
                                                    on:blur=move |_| {custom_money_is_focused.set(false)}
                                                />
                                                <a href="#" class="bg-emerald-600 text-white rounded-full p-5" on:click=move |_| on_custom_money_button_click(true, custom_money_change, &args2)>
                                                    <div class="pad-5 text-center">
                                                        "+"
                                                    </div>
                                                </a>
                                            </div>
                                            <div class="grid grid-cols-3 gap-5 rounded-[10px]">
                                                {change_money_button(-50, args.clone())}
                                                {change_money_button(-100, args.clone())}
                                                {change_money_button(-200, args.clone())}
                                                {change_money_button(-500, args.clone())}
                                                {change_money_button(-1000, args.clone())}
                                                {change_money_button(-2000, args.clone())}
                                                {change_money_button(-5000, args.clone())}

                                            </div>
                                        </div>
                                    </div>
                                </div>
                                <div class="pt-5">
                                    <ShowTransactions arguments=args.clone()/>
                                </div>
                                <div class="flex w-full justify-end pt-3">
                                    <a href=format!("/user/{}/transactions", user_id) class="bg-gray-400 rounded">
                                        <span class="text-white">"More Transactions"</span>
                                        <button class="w-full">
                                            <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><g id="SVGRepo_bgCarrier" stroke-width="0"></g><g id="SVGRepo_tracerCarrier" stroke-linecap="round" stroke-linejoin="round"></g><g id="SVGRepo_iconCarrier"> <g id="Menu / More_Grid_Big"> <g id="Vector"> <path d="M17 18C17 18.5523 17.4477 19 18 19C18.5523 19 19 18.5523 19 18C19 17.4477 18.5523 17 18 17C17.4477 17 17 17.4477 17 18Z" stroke="#9a9996" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"></path> <path d="M11 18C11 18.5523 11.4477 19 12 19C12.5523 19 13 18.5523 13 18C13 17.4477 12.5523 17 12 17C11.4477 17 11 17.4477 11 18Z" stroke="#9a9996" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"></path> <path d="M5 18C5 18.5523 5.44772 19 6 19C6.55228 19 7 18.5523 7 18C7 17.4477 6.55228 17 6 17C5.44772 17 5 17.4477 5 18Z" stroke="#9a9996" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"></path> <path d="M17 12C17 12.5523 17.4477 13 18 13C18.5523 13 19 12.5523 19 12C19 11.4477 18.5523 11 18 11C17.4477 11 17 11.4477 17 12Z" stroke="#9a9996" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"></path> <path d="M11 12C11 12.5523 11.4477 13 12 13C12.5523 13 13 12.5523 13 12C13 11.4477 12.5523 11 12 11C11.4477 11 11 11.4477 11 12Z" stroke="#9a9996" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"></path> <path d="M5 12C5 12.5523 5.44772 13 6 13C6.55228 13 7 12.5523 7 12C7 11.4477 6.55228 11 6 11C5.44772 11 5 11.4477 5 12Z" stroke="#9a9996" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"></path> <path d="M17 6C17 6.55228 17.4477 7 18 7C18.5523 7 19 6.55228 19 6C19 5.44772 18.5523 5 18 5C17.4477 5 17 5.44772 17 6Z" stroke="#9a9996" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"></path> <path d="M11 6C11 6.55228 11.4477 7 12 7C12.5523 7 13 6.55228 13 6C13 5.44772 12.5523 5 12 5C11.4477 5 11 5.44772 11 6Z" stroke="#9a9996" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"></path> <path d="M5 6C5 6.55228 5.44772 7 6 7C6.55228 7 7 6.55228 7 6C7 5.44772 6.55228 5 6 5C5.44772 5 5 5.44772 5 6Z" stroke="#9a9996" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"></path> </g> </g> </g></svg>
                                        </button>
                                    </a>
                                </div>
                            }.into_any()
                        }
                    }
                </div>
                </Suspense>
            }
        }.into_any()
        }
    }
    .into_any();
}

#[component]
pub fn SettingsIcon() -> impl IntoView {
    view! {
        <svg width="50px" height="50px" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><g id="SVGRepo_bgCarrier" stroke-width="0"></g><g id="SVGRepo_tracerCarrier" stroke-linecap="round" stroke-linejoin="round"></g><g id="SVGRepo_iconCarrier"> <path d="M15 12C15 13.6569 13.6569 15 12 15C10.3431 15 9 13.6569 9 12C9 10.3431 10.3431 9 12 9C13.6569 9 15 10.3431 15 12Z" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="--darkreader-inline-stroke: var(--darkreader-text-000000, #e8e6e3);" data-darkreader-inline-stroke=""></path> <path d="M12.9046 3.06005C12.6988 3 12.4659 3 12 3C11.5341 3 11.3012 3 11.0954 3.06005C10.7942 3.14794 10.5281 3.32808 10.3346 3.57511C10.2024 3.74388 10.1159 3.96016 9.94291 4.39272C9.69419 5.01452 9.00393 5.33471 8.36857 5.123L7.79779 4.93281C7.3929 4.79785 7.19045 4.73036 6.99196 4.7188C6.70039 4.70181 6.4102 4.77032 6.15701 4.9159C5.98465 5.01501 5.83376 5.16591 5.53197 5.4677C5.21122 5.78845 5.05084 5.94882 4.94896 6.13189C4.79927 6.40084 4.73595 6.70934 4.76759 7.01551C4.78912 7.2239 4.87335 7.43449 5.04182 7.85566C5.30565 8.51523 5.05184 9.26878 4.44272 9.63433L4.16521 9.80087C3.74031 10.0558 3.52786 10.1833 3.37354 10.3588C3.23698 10.5141 3.13401 10.696 3.07109 10.893C3 11.1156 3 11.3658 3 11.8663C3 12.4589 3 12.7551 3.09462 13.0088C3.17823 13.2329 3.31422 13.4337 3.49124 13.5946C3.69158 13.7766 3.96395 13.8856 4.50866 14.1035C5.06534 14.3261 5.35196 14.9441 5.16236 15.5129L4.94721 16.1584C4.79819 16.6054 4.72367 16.829 4.7169 17.0486C4.70875 17.3127 4.77049 17.5742 4.89587 17.8067C5.00015 18.0002 5.16678 18.1668 5.5 18.5C5.83323 18.8332 5.99985 18.9998 6.19325 19.1041C6.4258 19.2295 6.68733 19.2913 6.9514 19.2831C7.17102 19.2763 7.39456 19.2018 7.84164 19.0528L8.36862 18.8771C9.00393 18.6654 9.6942 18.9855 9.94291 19.6073C10.1159 20.0398 10.2024 20.2561 10.3346 20.4249C10.5281 20.6719 10.7942 20.8521 11.0954 20.94C11.3012 21 11.5341 21 12 21C12.4659 21 12.6988 21 12.9046 20.94C13.2058 20.8521 13.4719 20.6719 13.6654 20.4249C13.7976 20.2561 13.8841 20.0398 14.0571 19.6073C14.3058 18.9855 14.9961 18.6654 15.6313 18.8773L16.1579 19.0529C16.605 19.2019 16.8286 19.2764 17.0482 19.2832C17.3123 19.2913 17.5738 19.2296 17.8063 19.1042C17.9997 18.9999 18.1664 18.8333 18.4996 18.5001C18.8328 18.1669 18.9994 18.0002 19.1037 17.8068C19.2291 17.5743 19.2908 17.3127 19.2827 17.0487C19.2759 16.8291 19.2014 16.6055 19.0524 16.1584L18.8374 15.5134C18.6477 14.9444 18.9344 14.3262 19.4913 14.1035C20.036 13.8856 20.3084 13.7766 20.5088 13.5946C20.6858 13.4337 20.8218 13.2329 20.9054 13.0088C21 12.7551 21 12.4589 21 11.8663C21 11.3658 21 11.1156 20.9289 10.893C20.866 10.696 20.763 10.5141 20.6265 10.3588C20.4721 10.1833 20.2597 10.0558 19.8348 9.80087L19.5569 9.63416C18.9478 9.26867 18.6939 8.51514 18.9578 7.85558C19.1262 7.43443 19.2105 7.22383 19.232 7.01543C19.2636 6.70926 19.2003 6.40077 19.0506 6.13181C18.9487 5.94875 18.7884 5.78837 18.4676 5.46762C18.1658 5.16584 18.0149 5.01494 17.8426 4.91583C17.5894 4.77024 17.2992 4.70174 17.0076 4.71872C16.8091 4.73029 16.6067 4.79777 16.2018 4.93273L15.6314 5.12287C14.9961 5.33464 14.3058 5.0145 14.0571 4.39272C13.8841 3.96016 13.7976 3.74388 13.6654 3.57511C13.4719 3.32808 13.2058 3.14794 12.9046 3.06005Z" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="--darkreader-inline-stroke: var(--darkreader-text-000000, #e8e6e3);" data-darkreader-inline-stroke=""></path> </g></svg>
    }
}

fn change_money_button(
    money: i64,
    args: Rc<MoneyArgs>
) -> impl IntoView {
    view! {
        <a
            on:click=move |_| change_money_logic(money.into(), args.clone())
            href="#"
            class="p-5 text-white rounded-[10px] text-center text-[1.25em]"
            class=("bg-emerald-600", move || money > 0)
            class=("bg-red-400", move || money < 0)
        >{Money::format_eur_diff_value(money)}</a>
    }
}

fn change_money_logic(money: Money, args: Rc<MoneyArgs>){
    let user_id = args.user_id.clone();
    let money_signal = args.money;
    let error = args.error;
    let transactions = args.transactions;

    change_money_logic_raw(money, user_id, money_signal, error, transactions);
}

fn change_money_logic_raw(money: Money, user_id: i64, money_signal: RwSignal<Money>, error_signal: RwSignal<String>, transaction_signal: RwSignal<Vec<Transaction>>){
    spawn_local(async move {
        let t_type = if money.value > 0 { TransactionType::DEPOSIT } else { TransactionType::WITHDRAW };
        
        let resp = create_transaction(user_id, money.clone(), t_type).await;

        if resp.is_ok() {
            money_signal.update(|money_struct| (*money_struct).value = money_struct.value + money.value);
            error_signal.set(String::new());
            let new_transaction = resp.unwrap();
            transaction_signal.write().insert(0, new_transaction.into());
        } else {
            let error = resp.err().unwrap().to_string();

            error_signal.set(error);

        }
    })
}

fn on_custom_money_button_click(add: bool, value: RwSignal<String>, args: &MoneyArgs){
    let string = value.get_untracked();

    let error_signal = args.error;
    error_signal.set(String::new());

    if string.len() == 0 {
        return;
    }

    let mut money: Money = match string.try_into() {
        Ok(value) => value,
        Err(e) => {
            error_signal.set(format!("Failed to parse money: {}", e.to_string()));
            return;
        }
    };

    if money.value == 0 {
        return;
    }

    if !add {
        money.value = -money.value;
    }

    change_money_logic_raw(money, args.user_id, args.money, args.error, args.transactions);

    value.set(String::new());    
}
