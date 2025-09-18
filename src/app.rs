use std::collections::HashMap;

use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use thaw::ssr::SSRMountStyleProvider;

use crate::routes::{self, error::ErrorDisplay, state::FrontendStore};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <SSRMountStyleProvider>
            <!DOCTYPE html>
            <html lang="en">
                <head>
                    <meta charset="utf-8" />
                    <meta name="viewport" content="width=device-width, initial-scale=1" />
                    <AutoReload options=options.clone() />
                    <HydrationScripts options />
                    <MetaTags />
                </head>
                <body class="bg-[#25333f]">
                    <App />
                </body>
            </html>
        </SSRMountStyleProvider>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    use reactive_stores::Store;
    let store = FrontendStore {
        cached_sounds: HashMap::new(),
        error: RwSignal::default(),
    };
    provide_context(Store::new(store));
    let audio_ref = NodeRef::<leptos::html::Audio>::new();
    provide_context(audio_ref);

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/strichliste-rs.css" />

        // sets the document title
        <Title text="Strichliste-rs" />

        {routes::navbar::View()}
        <ErrorDisplay />
        <audio node_ref=audio_ref />

        // content for this welcome page
        <Router>
            <Routes fallback=|| {
                view! { <h1 class="text-white text-center bg-red-400">"Page not found!"</h1> }
            }>
                <Route path=path!("/") view=routes::home::View />
                <Route path=path!("/user/create") view=routes::user::Create />
                <Route path=path!("/user/:id") view=routes::user::ShowUser />
                <Route path=path!("/user/:id/settings") view=routes::user::settings::Show />
                <Route
                    path=path!("/user/:id/transactions")
                    view=routes::user::extra_transactions::Show
                />
                <Route path=path!("/user/:id/send_money") view=routes::user::send_money::Show />
                <Route path=path!("/articles") view=routes::articles::View />
                <Route path=path!("/articles/create") view=routes::articles::Create />
                <Route path=path!("/articles/:article_id") view=routes::articles::Edit />
            </Routes>
        </Router>
    }
}
