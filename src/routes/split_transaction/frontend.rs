use leptos::prelude::*;

use crate::routes::split_transaction::components::single_user_selection::SelectSingleUser;

#[component]
pub fn Show() -> impl IntoView {
    let primary_user = RwSignal::new(String::new());
    view! {
        <SelectSingleUser
            title=String::from("Who are you?")
            input=primary_user
        >
            <p class="text-white">"Hello"</p>
        </SelectSingleUser>
    }
}
