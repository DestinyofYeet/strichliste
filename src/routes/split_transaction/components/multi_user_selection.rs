use leptos::prelude::*;

use crate::routes::components::icons::DeleteIcon;

#[component]
pub fn MultiUserSelection(users_input: RwSignal<Vec<String>>) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center pt-5 gap-10 text-[1.25em]">
            <table class="w-full text-white border-collapse border-spacing-5">
                <tr class="bg-black">
                    <th class="pl-2">"Barcodes"</th>
                    <th></th>
                </tr>
                {move || {
                    let users_input_value = users_input
                        .get();
                    users_input_value
                        .into_iter()
                        .map(|user| {
                            view! {
                                <tr class="even:bg-gray-700 odd:bg-gray-500 text-center">
                                    <td class="px-2">
                                        <p>{user.clone()}</p>
                                    </td>
                                    <td class="px-2">
                                        <button
                                            class="size-8 pt-2"
                                            on:click=move |_| {
                                                users_input
                                                    .update(|vec| {
                                                        _ = vec
                                                            .remove(
                                                                vec
                                                                    .iter()
                                                                    .position(|elem| *elem == user)
                                                                    .expect("element should be in list!"),
                                                            );
                                                    });
                                            }
                                        >
                                            <DeleteIcon/>
                                        </button>
                                    </td>
                                </tr>
                            }
                        })
                        .collect_view()
                }}
            </table>
        </div>
    }
}
