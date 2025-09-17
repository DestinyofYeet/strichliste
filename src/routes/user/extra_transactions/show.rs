use leptos::{html, prelude::*};
use leptos_router::hooks::use_params_map;

use crate::{
    models::{Money, PageRequestParams, Transaction, UserId},
    routes::user::components::transaction_view::{format_transaction, get_user_transactions},
};

#[component]
pub fn Show() -> impl IntoView {
    let params = use_params_map();
    let user_id_string = params.read_untracked().get("id").unwrap_or_default();

    let user_id = match user_id_string.parse::<i64>() {
        Ok(value) => UserId(value),
        Err(_) => {
            return view! { <p class="text-red-400">"Invalid user id"</p> }.into_any();
        }
    };

    let transactions_per_page = 15;

    let page_count = RwSignal::new(0);
    let trans_resource = Resource::new(
        move || (page_count.get(), user_id, transactions_per_page),
        |(page_count, user_id, transactions_per_page)| {
            get_user_transactions(
                user_id,
                PageRequestParams {
                    offset: page_count * transactions_per_page,
                    limit: transactions_per_page,
                },
            )
        },
    );

    let money_signal = RwSignal::new(Money::default());

    let go_back_padding = "p-5";

    let transaction_signal = RwSignal::new(Vec::<Transaction>::new());

    view! {
        <div>
            <div class=go_back_padding>
                <button class=go_back_padding>
                    <a
                        href=format!("/user/{}", user_id)
                        class=format!("text-white bg-gray-400 rounded {}", go_back_padding)
                    >
                        "Go back"
                    </a>
                </button>
            </div>
            <ShowNavigationButtons
                page_count=page_count
                transaction_signal=transaction_signal
                transactions_per_page=transactions_per_page
            />
            <Transition fallback=move || {
                view! { <h1 class="text-white text-center">"Loading transactions!"</h1> }
            }>
                {move || match trans_resource.get() {
                    None => ().into_any(),
                    Some(value) => {
                        match value {
                            Err(e) => {
                                view! {
                                    <p class="text-red-400">
                                        "Failed to fetch transactions: "{e.to_string()}
                                    </p>
                                }
                                    .into_any()
                            }
                            Ok(value) => {
                                transaction_signal
                                    .update(|transactions| *transactions = value.items);

                                view! {
                                    {transaction_signal
                                        .get()
                                        .iter()
                                        .map(|transaction| {
                                            format_transaction(transaction, user_id, money_signal)
                                        })
                                        .collect_view()}
                                }
                                    .into_any()
                            }
                        }
                    }
                }}
            </Transition>
        </div>
        <ShowNavigationButtons
            page_count=page_count
            transaction_signal=transaction_signal
            transactions_per_page=transactions_per_page
        />
    }
    .into_any()
}

#[component]
pub fn ShowNavigationButtons(
    page_count: RwSignal<usize>,
    transaction_signal: RwSignal<Vec<Transaction>>,
    transactions_per_page: usize,
) -> impl IntoView {
    view! {
        <div class="flex justify-between p-2">
            {
                let button_ref_prev = NodeRef::<html::Button>::new();
                view! {
                    {move || match transaction_signal.get().len() {
                        0 => {
                            button_ref_prev
                                .get()
                                .map(|elem| { elem.set_attribute("disabled", "") });
                        }
                        _ => {
                            button_ref_prev.get().map(|elem| { elem.remove_attribute("disabled") });
                        }
                    }}
                    <button
                        class="rounded p-5"
                        class=(["bg-gray-400", "text-white"], move || page_count.get() != 0)
                        class=(["bg-white", "text-black"], move || page_count.get() == 0)
                        node_ref=button_ref_prev
                        on:click=move |_| {
                            page_count
                                .update(|value| {
                                    if *value != 0 {
                                        *value -= 1;
                                    }
                                });
                        }
                    >
                        "Previous page"
                    </button>
                    <button
                        class="rounded p-5"
                        class=(
                            ["bg-gray-400", "text-white"],
                            move || transaction_signal.get().len() == transactions_per_page,
                        )
                        class=(
                            ["bg-white", "text-black"],
                            move || transaction_signal.get().len() != transactions_per_page,
                        )
                        on:click=move |_| {
                            let transaction_count = transaction_signal.get_untracked().len();
                            if transaction_count == 0 {
                                return;
                            }
                            if transaction_count < transactions_per_page {
                                return;
                            }
                            page_count.update(|value| *value += 1);
                        }
                    >
                        "Next page"
                    </button>
                }
            }
        </div>
    }
}
