use leptos::{
    html::{self},
    prelude::*,
    task::spawn_local,
};
#[cfg(feature = "ssr")]
use {
    crate::models::Money,
    tracing::{debug, error},
};

use crate::models::{Article, Barcode, BarcodeDiff};

#[server]
pub async fn get_article(article_id: i64) -> Result<Article, ServerFnError> {
    use crate::backend::core::ServerState;
    let state: ServerState = expect_context();
    use axum::http::StatusCode;
    use leptos_axum::ResponseOptions;

    let response_opts: ResponseOptions = expect_context();

    let article = Article::get(&*state.db.lock().await, article_id).await;

    let article = match article {
        Ok(value) => value,
        Err(e) => {
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            return Err(ServerFnError::new(format!(
                "Error getting article from db: {}",
                e
            )));
        }
    };

    match article {
        None => {
            response_opts.set_status(StatusCode::BAD_REQUEST);
            Err(ServerFnError::new(format!(
                "Unknown Article id '{}'",
                article_id
            )))
        }

        Some(value) => Ok(value),
    }
}

#[server]
pub async fn update_article(
    id: i64,
    name: String,
    cost: String,
    barcodes: Option<Vec<BarcodeDiff>>,
) -> Result<(), ServerFnError> {
    use crate::backend::core::ServerState;
    let state: ServerState = expect_context();
    use axum::http::StatusCode;
    use leptos_axum::redirect;
    use leptos_axum::ResponseOptions;

    let response_opts: ResponseOptions = expect_context();

    let mut article = get_article(id).await?;

    let cost: Money = match cost.clone().try_into() {
        Ok(value) => value,
        Err(e) => {
            response_opts.set_status(StatusCode::BAD_REQUEST);
            return Err(ServerFnError::new(format!(
                "Failed to convert '{}' to internal money representation: {}",
                cost, e
            )));
        }
    };

    let db = &*state.db.lock().await;

    let mut db_transaction = match db.get_conn_transaction().await {
        Ok(value) => value,
        Err(e) => {
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            error!("Failed to get transaction for database: {}", e);
            return Err(ServerFnError::new("Failed to get transaction handle!"));
        }
    };

    if article.name != name {
        match article
            .set_name(&mut *db_transaction, name.trim().to_string())
            .await
        {
            Ok(_) => {}
            Err(e) => {
                response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
                error!("Failed to update article name: {}", e);
                return Err(ServerFnError::new(format!(
                    "Failed to update article name: {}",
                    e
                )));
            }
        }
    }

    debug!(
        "Old money: {} | New money: {}",
        article.cost.value, cost.value
    );

    if article.cost.value != cost.value {
        match article.set_cost(&mut *db_transaction, cost).await {
            Ok(_) => {}
            Err(e) => {
                response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
                error!("Failed to update article cost: {}", e);
                return Err(ServerFnError::new("FAiled to update article cost"));
            }
        }
    }

    match barcodes {
        None => {}
        Some(barcodes) => {
            let result = article.set_barcodes(&mut *db_transaction, barcodes).await;

            if let Err(e) = result {
                response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
                error!("Failed to set barcodes: {}", e);
                return Err(ServerFnError::new(format!("Failed to set barcodes: {}", e)));
            }
        }
    }

    match db_transaction.commit().await {
        Ok(_) => {}
        Err(e) => {
            response_opts.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            error!("Failed to commit transaction: {}", e);
            return Err(ServerFnError::new("Failed to commit transaction"));
        }
    }

    redirect("/articles");

    Ok(())
}

#[component]
pub fn Edit() -> impl IntoView {
    use leptos_router::hooks::use_params_map;
    let params = use_params_map();
    let article_id_string = params
        .read_untracked()
        .get("article_id")
        .unwrap_or_default();

    let article_id = article_id_string.parse::<i64>();
    let article_id = match article_id {
        Ok(value) => value,
        Err(err) => {
            return view! {
                <p class="text-red-400 text-center">
                    "Failed to get id from params map: "{err.to_string()}
                </p>
            }
            .into_any();
        }
    };

    let article_resource = OnceResource::new(get_article(article_id));

    view! {
        {move || match article_resource.get() {
            None => view! { <p class="text-white text-center">"Loading article..."</p> }.into_any(),
            Some(value) => {
                let article = match value {
                    Ok(value) => value,
                    Err(e) => {
                        let error_msg = match e {
                            ServerFnError::ServerError(msg) => msg,
                            _ => e.to_string(),
                        };
                        return view! {
                            <p class="text-red-400 text-center">
                                "Failed to load Article: "{error_msg}
                            </p>
                        }
                            .into_any();
                    }
                };

                view! { <SingleArticleView article /> }
                    .into_any()
            }
        }}
    }
    .into_any()
}

#[component]
fn SingleArticleView(article: Article) -> impl IntoView {
    let name_node = NodeRef::<html::Input>::new();
    let cost_node = NodeRef::<html::Input>::new();

    let new_barcode_node = NodeRef::<html::Input>::new();

    let barcodes_signal = RwSignal::new(article.barcodes.clone());
    let barcodes_diff_signal = RwSignal::new(Vec::<BarcodeDiff>::new());

    let error_signal = RwSignal::new(String::new());

    let class_css = "flex flex-col gap-5";
    let input_css = "ml-5 text-black rounded-[5px] text-center";

    let clone = article.clone();

    let on_click = move |_| {
        let mut article = clone.clone();
        article.name = name_node.get().unwrap().value();
        // console_log("Hello");

        spawn_local(async move {
            let Article {
                id,
                name: _,
                cost: _,
                sounds: _,
                barcodes: _,
            } = article;

            let name = name_node
                .get_untracked()
                .expect("name input should be mounted")
                .value();
            let cost = cost_node
                .get_untracked()
                .expect("name input should be mounted")
                .value();

            let barcodes = barcodes_diff_signal.get_untracked();
            if let Err(e) = update_article(id, name, cost, Some(barcodes)).await {
                let msg = match e {
                    ServerFnError::ServerError(msg) => msg,
                    _ => e.to_string(),
                };

                error_signal.set(msg);
            }
        });
    };
    return view! {
        {move || {
            let msg = error_signal.get();
            match msg.len() {
                0 => ().into_any(),
                _ => {

                    view! {
                        <div class="bg-red-400 p-5">
                            <p class="text-white text-center">"Failed to update article: "{msg}</p>
                        </div>
                    }
                        .into_any()
                }
            }
        }}
        <div class="flex flex-col items-center pt-5 gap-10 text-[1.25em]">
            <div class="flex justify-center pt-5">
                <div class=format!("{} items-end", { class_css })>
                    <a class="text-white">"Name:"</a>
                    <a class="text-white">"Cost:"</a>

                </div>
                <div class=format!("{} items-center", { class_css })>
                    <input class=input_css type="text" value=article.name node_ref=name_node />
                    <input
                        class=input_css
                        type="text"
                        value=article.cost.format()
                        node_ref=cost_node
                    />
                </div>
            </div>
            <div>
                <table class="w-full text-white border-collapse border-spacing-5">
                    <tr class="bg-black">
                        <th class="pl-2">"Barcodes"</th>
                        <th></th>
                    </tr>
                    {move || {
                        barcodes_signal
                            .get()
                            .iter()
                            .map(|barcode| {
                                let code = barcode.clone().0;
                                view! {
                                    <tr class="even:bg-gray-700 odd:bg-gray-500 text-center">
                                        <td class="px-2">
                                            <p>{code.clone()}</p>
                                        </td>
                                        <td class="px-2">
                                            <button
                                                class="size-8 pt-2"
                                                on:click=move |_| {
                                                    barcodes_signal
                                                        .update(|vec| {
                                                            _ = vec
                                                                .remove(
                                                                    vec
                                                                        .iter()
                                                                        .position(|elem| elem.0 == code)
                                                                        .expect("element should be in list!"),
                                                                );
                                                        });
                                                    barcodes_diff_signal
                                                        .write()
                                                        .push(BarcodeDiff::Removed(code.clone()));
                                                }
                                            >
                                                <svg
                                                    viewBox="0 0 32 32"
                                                    version="1.1"
                                                    xmlns="http://www.w3.org/2000/svg"
                                                    xmlns:xlink="http://www.w3.org/1999/xlink"
                                                    xmlns:sketch="http://www.bohemiancoding.com/sketch/ns"
                                                    fill="#ed333b"
                                                    style="--darkreader-inline-fill: var(--darkreader-background-ed333b, #a90f16);"
                                                    data-darkreader-inline-fill=""
                                                >
                                                    <g id="SVGRepo_bgCarrier" stroke-width="0"></g>
                                                    <g
                                                        id="SVGRepo_tracerCarrier"
                                                        stroke-linecap="round"
                                                        stroke-linejoin="round"
                                                    ></g>
                                                    <g id="SVGRepo_iconCarrier">
                                                        <title>cross-circle</title>
                                                        <desc>Created with Sketch Beta.</desc>
                                                        <defs></defs>
                                                        <g
                                                            id="Page-1"
                                                            stroke="none"
                                                            stroke-width="1"
                                                            fill="none"
                                                            fill-rule="evenodd"
                                                            sketch:type="MSPage"
                                                            style="--darkreader-inline-stroke: none;"
                                                            data-darkreader-inline-stroke=""
                                                        >
                                                            <g
                                                                id="Icon-Set"
                                                                sketch:type="MSLayerGroup"
                                                                transform="translate(-568.000000, -1087.000000)"
                                                                fill="#ed333b"
                                                                style="--darkreader-inline-fill: var(--darkreader-background-000000, #ed333b);"
                                                                data-darkreader-inline-fill=""
                                                            >
                                                                <path
                                                                    d="M584,1117 C576.268,1117 570,1110.73 570,1103 C570,1095.27 576.268,1089 584,1089 C591.732,1089 598,1095.27 598,1103 C598,1110.73 591.732,1117 584,1117 L584,1117 Z M584,1087 C575.163,1087 568,1094.16 568,1103 C568,1111.84 575.163,1119 584,1119 C592.837,1119 600,1111.84 600,1103 C600,1094.16 592.837,1087 584,1087 L584,1087 Z M589.717,1097.28 C589.323,1096.89 588.686,1096.89 588.292,1097.28 L583.994,1101.58 L579.758,1097.34 C579.367,1096.95 578.733,1096.95 578.344,1097.34 C577.953,1097.73 577.953,1098.37 578.344,1098.76 L582.58,1102.99 L578.314,1107.26 C577.921,1107.65 577.921,1108.29 578.314,1108.69 C578.708,1109.08 579.346,1109.08 579.74,1108.69 L584.006,1104.42 L588.242,1108.66 C588.633,1109.05 589.267,1109.05 589.657,1108.66 C590.048,1108.27 590.048,1107.63 589.657,1107.24 L585.42,1103.01 L589.717,1098.71 C590.11,1098.31 590.11,1097.68 589.717,1097.28 L589.717,1097.28 Z"
                                                                    id="cross-circle"
                                                                    sketch:type="MSShapeGroup"
                                                                ></path>
                                                            </g>
                                                        </g>
                                                    </g>
                                                </svg>
                                            </button>
                                        </td>
                                    </tr>
                                }
                            })
                            .collect_view()
                    }}
                </table>
            </div>
            <div class="flex justify-center pt-5 gap-5">
                <input class="text-black rounded-[5px] text-center" node_ref=new_barcode_node />
                <div class="w-[10vw]">
                    <button
                        type="button"
                        class="w-full bg-indigo-700 hover:bg-pink-700 text-white font-bold py-2 px-4 mb-6 rounded"
                        on:click=move |_| {
                            let node = new_barcode_node
                                .get()
                                .expect("new_barcode_input should be mounted!");
                            let new_barcode = node.value();
                            if new_barcode.is_empty() {
                                return;
                            }
                            barcodes_signal.write().push(Barcode(new_barcode.clone()));
                            node.set_value("");
                            barcodes_diff_signal.write().push(BarcodeDiff::Added(new_barcode));
                        }
                    >
                        "Add Barcode"
                    </button>
                </div>
            </div>

            // Last element (Submit button)
            <div class="w-[30vw]">
                <input
                    class="w-full bg-indigo-700 hover:bg-pink-700 text-white font-bold py-2 px-4 mb-6 rounded"
                    type="submit"
                    value="Upate article"
                    on:click=on_click
                />
            </div>
        </div>
    }
    .into_any();
}
