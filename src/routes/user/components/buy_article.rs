use std::rc::Rc;

use leptos::{ev, html, leptos_dom::logging::console_log, prelude::*, task::spawn_local};

use crate::{
    backend::core::Article,
    models::{play_sound, Money, Transaction, UserId},
    routes::{articles::get_all_articles, user::MoneyArgs},
};

#[cfg(feature = "ssr")]
use {
    crate::{backend::database::DBGROUP_SNACKBAR_ID, models::Group, routes::articles::get_article},
    tracing::error,
};

#[server]
pub async fn get_articles_per_user(user_id: UserId) -> Result<Vec<Article>, ServerFnError> {
    use crate::backend::core::ServerState;
    let state: ServerState = expect_context();
    use axum::http::StatusCode;
    use leptos_axum::ResponseOptions;

    let response_opts: ResponseOptions = expect_context();

    let db = state.db.lock().await;

    match Article::get_articles_for_user(&db, user_id).await {
        Ok(value) => Ok(value),

        Err(e) => {
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            error!("Failed to fetch per user articles: {}", e);
            Err(ServerFnError::new("Failed to fetch per user articles!"))
        }
    }
}

#[server]
pub async fn buy_article_by_id(
    user_id: UserId,
    article_id: i64,
) -> Result<Transaction, ServerFnError> {
    use crate::backend::core::ServerState;
    let state: ServerState = expect_context();
    use axum::http::StatusCode;
    use leptos_axum::ResponseOptions;

    let response_opts: ResponseOptions = expect_context();

    let article = get_article(article_id).await?;

    let db = state.db.lock().await;
    let mut db_trans = match db.get_conn_transaction().await {
        Ok(value) => value,
        Err(e) => {
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            error!("Failed to create db_transaction: {}", e);
            return Err(ServerFnError::new("Failed to create db connection"));
        }
    };

    let user_group = match Group::get_user_group_id(&mut *db_trans, user_id).await {
        Ok(value) => value,
        Err(e) => {
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            error!("Failed to get user single group: {}", e);
            return Err(ServerFnError::new(
                "Failed to find single user group for user",
            ));
        }
    };

    let transaction_id = Transaction::create(
        &mut *db_trans,
        user_group,
        DBGROUP_SNACKBAR_ID,
        crate::models::TransactionType::Bought(article_id),
        Some(article.name.clone()),
        article.cost,
        &state.settings,
    )
    .await?;

    let transaction = match Transaction::get(&mut *db_trans, transaction_id, user_id).await {
        Ok(Some(o)) => o,
        _ => {
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            error!("Failed to read back db_transaction");
            return Err(ServerFnError::new("Failed to read back db_transaction"));
        }
    };

    match db_trans.commit().await {
        Ok(_) => {}
        Err(e) => {
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            error!("Failed to commit transaction: {e}");
            return Err(ServerFnError::new("Failed to commit the db transaction!"));
        }
    }

    Ok(transaction)
}

pub fn buy_article(
    user_id: UserId,
    article_id: i64,
    money: RwSignal<Money>,
    error: RwSignal<String>,
    transactions: RwSignal<Vec<Transaction>>,
    audio_ref: NodeRef<leptos::html::Audio>,
) {
    console_log(&format!("Need to buy article with id: {article_id}"));
    let args = MoneyArgs {
        user_id,
        money,
        error,
        transactions,
        audio_ref,
    };
    spawn_local(async move {
        match buy_article_by_id(user_id, article_id).await {
            Ok(transaction) => {
                money.update(|money| money.value -= transaction.money.value);
                transactions.update(|trns| trns.insert(0, transaction));
                error.set(String::new());
                play_sound(
                    Rc::new(args),
                    crate::models::AudioPlayback::Bought(article_id),
                );
            }

            Err(e) => {
                error.set(format!("Failed to buy article: {e}"));
            }
        }
    });
}

#[component]
pub fn BuyArticle(args: Rc<MoneyArgs>) -> impl IntoView {
    let m_clone = args.clone();
    let MoneyArgs {
        user_id,
        money,
        error,
        transactions,
        audio_ref,
    } = *args;
    let personal_articles = OnceResource::new(get_articles_per_user(user_id));
    view! {
        <div>
            <Suspense fallback=move || {
                view! { <p class="text-center text-white">"Loading Articles"</p> }
            }>
                <div class="grid grid-cols-3 text-white text-center gap-2 text-[1.25em] p-2 pt-4">
                    {move || {
                        personal_articles
                            .get()
                            .map(|article| {
                                let article = match article {
                                    Ok(value) => {
                                        value
                                            .into_iter()
                                            .take(9)
                                            .collect::<Vec<Article>>()
                                    }
                                    Err(e) => {
                                        let msg = match e {
                                            ServerFnError::ServerError(msg) => msg,
                                            _ => e.to_string(),
                                        };
                                        return view! {
                                            <p class="bg-red-400 text-white text-center">
                                                {format!("Failed to fetch articles: {}", msg)}
                                            </p>
                                        }
                                            .into_any();
                                    }
                                };
                                article
                                    .into_iter()
                                    .map(|article| {
                                        let Article { id, name, cost, sounds: _, barcodes: _ } = article;

                                        view! {
                                            <button
                                                class="bg-gray-700 rounded p-2"
                                                on:click=move |_| {
                                                    buy_article(
                                                        user_id,
                                                        id,
                                                        money,
                                                        error,
                                                        transactions,
                                                        audio_ref,
                                                    );
                                                }
                                            >
                                                <div>{name}" | "{cost.format_eur()}</div>
                                            </button>
                                        }
                                    })
                                    .collect_view()
                                    .into_any()
                            })
                    }}
                </div>
            </Suspense>
            <ArticleSearch money_args=m_clone.clone() />
        </div>
    }
}

#[component]
pub fn ArticleSearch(money_args: Rc<MoneyArgs>) -> impl IntoView {
    let MoneyArgs {
        user_id,
        money,
        error,
        transactions,
        audio_ref,
    } = *money_args;
    let articles_resource = OnceResource::new(get_all_articles(None));

    let dropdown_div = NodeRef::<html::Div>::new();
    let search_term = RwSignal::new(String::new());
    let filtered_articles = RwSignal::new(Vec::<Article>::new());

    let articles_signal = RwSignal::new(Vec::<Article>::new());

    let on_input = move |_ev: ev::Event| {
        match search_term.get().len() {
            0 => filtered_articles.set(Vec::<Article>::new()),
            _ => filtered_articles.update(|val| {
                *val = articles_signal
                    .get()
                    .iter()
                    .filter(|elem| {
                        elem.name
                            .to_lowercase()
                            .contains(&search_term.get().to_lowercase())
                    })
                    .take(5)
                    .cloned()
                    .collect::<Vec<Article>>();
            }),
        };
    };

    view! {
        {move || {
            articles_resource
                .get()
                .map(|value| {
                    match value {
                        Ok(value) => {
                            articles_signal.set(value);
                            ().into_any()
                        }
                        Err(e) => {
                            let msg = match e {
                                ServerFnError::ServerError(msg) => msg,
                                _ => e.to_string(),
                            };

                            view! {
                                <p class="bg-red-400 text-white text-center">
                                    {format!("Failed to fetch articles: {}", msg)}
                                </p>
                            }
                                .into_any()
                        }
                    }
                })
        }}
        <div class="w-full min-w-[200px] flex flex-col items-center p-2">
            <div class="relative">
                <input
                    class="text-white w-full bg-transparent placeholder:text-slate-400 text-slate-700 text-sm border border-slate-200 rounded-md pl-3 pr-28 py-2 transition duration-300 ease focus:outline-none focus:border-slate-400 hover:border-slate-300 shadow-sm focus:shadow"
                    placeholder="Search for articles"
                    autocomplete=false
                    bind:value=search_term
                    on:input=on_input
                />
                <button
                    class="absolute top-1 right-1 flex items-center rounded bg-slate-800 py-1 px-2.5 border border-transparent text-center text-sm text-white transition-all shadow-sm hover:shadow focus:bg-slate-700 focus:shadow-none active:bg-slate-700 hover:bg-slate-700 active:shadow-none disabled:pointer-events-none disabled:opacity-50 disabled:shadow-none"
                    type="button"
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 24 24"
                        fill="currentColor"
                        class="w-4 h-4 mr-2"
                    >
                        <path
                            fill-rule="evenodd"
                            d="M10.5 3.75a6.75 6.75 0 1 0 0 13.5 6.75 6.75 0 0 0 0-13.5ZM2.25 10.5a8.25 8.25 0 1 1 14.59 5.28l4.69 4.69a.75.75 0 1 1-1.06 1.06l-4.69-4.69A8.25 8.25 0 0 1 2.25 10.5Z"
                            clip-rule="evenodd"
                        />
                    </svg>

                    Search
                </button>
            </div>
            <div node_ref=dropdown_div class=("hidden", move || search_term.get().is_empty())>
                {move || {
                    filtered_articles
                        .get()
                        .into_iter()
                        .map(|elem| {
                            view! {
                                <button on:click=move |_| {
                                    buy_article(
                                        user_id,
                                        elem.id,
                                        money,
                                        error,
                                        transactions,
                                        audio_ref,
                                    );
                                    search_term.set(String::new());
                                }>
                                    <div class="p-2 m-2 rounded text-white bg-gray-700">
                                        <p>{elem.name.clone()}" | "{elem.cost.format_eur()}</p>
                                    </div>
                                </button>
                            }
                                .into_any()
                        })
                        .collect_view()
                }}
            </div>
        </div>
    }.into_any()
}
