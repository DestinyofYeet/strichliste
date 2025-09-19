use leptos::prelude::*;
#[cfg(feature = "ssr")]
use tracing::error;

use crate::backend::core::Article;

#[server]
pub async fn get_article_by_barcode(barcode: String) -> Result<Option<Article>, ServerFnError> {
    use crate::backend::core::ServerState;
    let state: ServerState = expect_context();
    use axum::http::StatusCode;
    use leptos_axum::ResponseOptions;

    let response_opts: ResponseOptions = expect_context();

    let db = state.db.lock().await;

    match Article::get_by_barcode(&db, barcode).await {
        Err(e) => {
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            Err(ServerFnError::new(format!(
                "Failed to get article from db: {}",
                e
            )))
        }

        Ok(value) => Ok(value),
    }
}

#[server]
pub async fn get_all_articles(limit: Option<i64>) -> Result<Vec<Article>, ServerFnError> {
    use crate::backend::core::ServerState;
    let state: ServerState = expect_context();
    use axum::http::StatusCode;
    use leptos_axum::ResponseOptions;

    let response_opts: ResponseOptions = expect_context();

    let articles = Article::get_all(&*state.db.lock().await, limit).await;
    articles.map_err(|e| {
        let err = e.to_string();
        error!("Could not fetch articles {}", err);
        response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
        ServerFnError::new(err)
    })
}

#[component]
pub fn View() -> impl IntoView {
    view! {
        <div class="grid grid-cols-10 gap-10 py-10">
            <div class="col-span-1 pl-5 justify-self-center">
                <a href="/articles/create" class="inline-block">
                    <div class="flex justify-center">
                        // joinked from: https://gist.github.com/ibelick/0c92c1aba54c2f7e8b3a7381426ed029
                        <button class="inline-flex h-10 w-10 items-center justify-center rounded-full bg-gray-50 text-black drop-shadow-sm transition-colors duration-150 hover:bg-gray-200">
                            "+"
                        </button>
                    </div>
                </a>
            </div>
            <div class="col-span-9 pr-7">
                <ShowArticles />
            </div>
        </div>
    }
}

#[component]
fn ShowArticles() -> impl IntoView {
    let all_articles = OnceResource::new(get_all_articles(None));
    view! {
        <Suspense fallback=move || {
            view! { <h1>"Loading articles..."</h1> }
        }>
            {move || {
                all_articles
                    .get()
                    .map(|articles| {
                        match articles {
                            Err(err) => {
                                let msg = match err {
                                    ServerFnError::ServerError(msg) => msg,
                                    _ => err.to_string(),
                                };
                                view! {
                                    <p class="text-red-900">"Failed to fetch article: " {msg}</p>
                                }
                                    .into_any()
                            }
                            Ok(mut articles) => {
                                view! {
                                    <table class="w-full text-white p-2">
                                        <thead>
                                            <tr class="bg-black">
                                                <th>"Name"</th>
                                                <th>"Preis"</th>
                                                <th></th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {
                                                articles.sort_by(|a, b| a.name.cmp(&b.name));
                                                articles
                                                    .into_iter()
                                                    .map(|article| {
                                                        view! {
                                                            <tr class="even:bg-gray-700 odd:bg-gray-500">
                                                                <td class="p-2 text-center">{article.name}</td>
                                                                <td class="p-2 text-center">{article.cost.format_eur()}</td>
                                                                <td class="bg-green-700 p-2">
                                                                    <a href=format!("/articles/{}", article.id)>
                                                                        <p class="text-center">"Edit"</p>
                                                                    </a>
                                                                </td>
                                                            </tr>
                                                        }
                                                    })
                                                    .collect_view()
                                            }
                                        </tbody>
                                    </table>
                                }
                                    .into_any()
                            }
                        }
                    })
            }}
        </Suspense>
    }
}
