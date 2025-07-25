use std::rc::Rc;

use chrono::Utc;
use leptos::{ev, leptos_dom::logging::console_log, prelude::*, task::spawn_local};

use crate::routes::{articles::get_article_by_barcode, user::MoneyArgs};

use super::buy_article::buy_article;

pub fn invisible_scan_input(
    is_focused_signal: RwSignal<bool>,
    error_signal: RwSignal<String>,
    money_args: Rc<MoneyArgs>,
    user_id: i64,
) -> impl IntoView {
    let input_signal = RwSignal::new(String::new());
    let last_input = RwSignal::new(Utc::now());

    let timediff = move || ((Utc::now() - last_input.get()).num_seconds() > 30);

    let handle = window_event_listener(ev::keypress, move |ev| match ev.key().as_str() {
        "Enter" => {
            if is_focused_signal.get() {
                return;
            }
            if timediff() {
                input_signal.write_only().set(String::new())
            }
            let scan_input = input_signal.read_untracked().clone();
            input_signal.write_only().set(String::new());

            if scan_input.len() == 0 {
                return;
            }

            let money_args_clone = money_args.clone();

            spawn_local(async move {
                console_log(&format!("Input {}", scan_input));
                let article = get_article_by_barcode(scan_input.clone()).await;

                let article = match article {
                    Ok(value) => value,
                    Err(e) => {
                        error_signal.set(format!("Failed to fetch article from server: {}", e));
                        return;
                    }
                };

                match article {
                    None => {
                        console_log(&format!(
                            "No article could be found with barcode '{}'",
                            scan_input
                        ));
                    }

                    Some(value) => {
                        console_log(&format!("Need to buy article: {}", value.name));
                        buy_article(
                            user_id,
                            value.id,
                            money_args_clone.money,
                            money_args_clone.error,
                            money_args_clone.transactions,
                        );
                    }
                }
            });
        }

        _ => {
            if is_focused_signal.get() {
                return;
            }

            // console_log(&format!("Seconds till last input: {} | Has passed 30s: {}", (Utc::now() - last_input.get()).num_seconds(), timediff()));

            // Clear input if nothing was typed for 30 seconds
            if timediff() {
                input_signal.write_only().set(String::new());
            }

            input_signal.update_untracked(|string| string.push_str(&ev.key()));

            last_input.write_only().set(Utc::now());
        }
    });

    on_cleanup(move || {
        handle.remove();
    });

    return view! {
        // {
        //     move || match is_focused_signal.get() {
        //         true => console_log("input is focused"),
        //         false => console_log("input is out of focus")
        //     }
        // }
    };
}
