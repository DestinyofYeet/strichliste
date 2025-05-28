use std::rc::Rc;

use chrono::{DateTime, Local, Timelike, Utc};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use tracing::{debug, error};

use crate::models::{Transaction, TransactionType, User};

use super::MoneyArgs;

#[server]
pub async fn get_user_transactions(
    user_id: i64,
    limit: i64,
) -> Result<Vec<Transaction>, ServerFnError> {
    use crate::backend::ServerState;
    let state: ServerState = expect_context();
    use axum::http::StatusCode;
    use leptos_axum::ResponseOptions;

    let response_opts: ResponseOptions = expect_context();

    let transactions =
        Transaction::get_user_transactions(&*state.db.lock().await, user_id, limit).await;

    if transactions.is_err() {
        error!(
            "Failed to fetch transactions: {}",
            transactions.err().unwrap()
        );
        response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
        return Err(ServerFnError::new("Failed to fetch transactions!"));
    }

    let transactions = transactions.unwrap();

    Ok(transactions)
}

#[server]
pub async fn undo_transaction(
    user_id: i64,
    transaction_id: i64,
) -> Result<(), ServerFnError> {
    debug!("Need to undo transaction {} for user {}", transaction_id, user_id);
    Ok(())
}

#[component]
pub fn ShowTransactions(arguments: Rc<MoneyArgs>) -> impl IntoView {
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

    let transaction_data = OnceResource::new(get_user_transactions(user_id, 10));

    let transaction_signal = arguments.transactions;

    return view! {
        <Suspense
            fallback=move || view!{<p class="text-white text-center p-5">"Loading transactions"</p>}
        >
        {
            move || {
                let transactions = transaction_data.get();

                if transactions.is_none() {
                    return view!{
                        <p class="text-white bg-red-400 text-center">"Failed to fetch transactions"</p>
                    }.into_any();
                }

                let transactions = transactions.unwrap();

                if transactions.is_err() {
                    let msg = match transactions.err().unwrap() {
                        ServerFnError::ServerError(msg) => msg,
                        _ => "Failed to fetch transactions".to_string()
                    };

                    return view! {
                        <p class="text-white text-center bg-red-400">"Failed to fetch users because: "{msg}</p>
                    }.into_any();
                }

                let mut transactions = transactions.unwrap();
                transactions.reverse(); // place new transaction at the top
                transaction_signal.write_untracked().append(&mut transactions);
                return view! {
                    <div class="pl-4 text-[1.25em]">
                        {
                             transaction_signal.get().iter().map(|transaction| {
                                format_transaction(transaction, user_id)
                            }).collect_view()           
                        }
                    </div>
                }
                .into_any();
            }
        }

        </Suspense>
    }
    .into_any();
}

pub fn format_transaction(transaction: &Transaction, user_id: i64) -> impl IntoView {
    // <svg width="50px" height="50px" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><g id="SVGRepo_bgCarrier" stroke-width="0"></g><g id="SVGRepo_tracerCarrier" stroke-linecap="round" stroke-linejoin="round"></g><g id="SVGRepo_iconCarrier"> <path opacity="0.5" d="M4 11.25C3.58579 11.25 3.25 11.5858 3.25 12C3.25 12.4142 3.58579 12.75 4 12.75V11.25ZM4 12.75H20V11.25H4V12.75Z" fill="#a5a4a8" style="--darkreader-inline-fill: var(--darkreader-background-a5a4a8, #161f3d);" data-darkreader-inline-fill=""></path> <path d="M14 6L20 12L14 18" stroke="#a5a4a8" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" style="--darkreader-inline-stroke: var(--darkreader-text-a5a4a8, #acc4e0);" data-darkreader-inline-stroke=""></path> </g></svg>
    let now: DateTime<Utc> = Utc::now();
    let diff = now - transaction.timestamp;

    let undo_action = ServerAction::<UndoTransaction>::new();
    let transaction_id = transaction.id.unwrap();
    return view! {
        <div class="grid grid-cols-3 items-center border-t-8 border-gray-300 p-2">
        {
            match transaction.t_type {
                TransactionType::DEPOSIT | TransactionType::WITHDRAW => view!{
                    <p class=""
                        class=("text-green-500", transaction.money >= 0)
                        class=("text-red-400", transaction.money < 0)
                    >{User::calc_money(transaction.money)}"€"</p>
                    <p></p>

                }.into_any(),

                _ => view!{}.into_any(),
            }
        }
        {
            // grace period for undoing transactions
            if diff.num_minutes() > 2 {
                view!{
                    <p class="text-white">{format!("{}", transaction.timestamp.with_timezone(&Local).format("%d.%m.%Y %H:%M:%S"))}</p>
                }.into_any()
            } else {
                view! {
                    <ActionForm action=undo_action>
                        <input type="hidden" name="user_id" value={user_id}/>
                        <input type="hidden" name="transaction_id" value={transaction_id}/>
                        <input type="submit" class="text-white" value="Undo"/>
                    </ActionForm>
                }.into_any()
            }
        }
        </div>
    };
}
