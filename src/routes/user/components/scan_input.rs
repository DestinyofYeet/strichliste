use std::rc::Rc;

use chrono::Utc;
use leptos::{ev, leptos_dom::logging::console_log, prelude::*, task::spawn_local};

use crate::routes::{articles::get_article_by_barcode, user::UserArgs};

use super::buy_article::buy_article;

pub fn invisible_scan_input(
    is_focused_signal: RwSignal<bool>,
    error_signal: RwSignal<String>,
    user_args: Rc<UserArgs>,
) -> impl IntoView {
    let input_signal = RwSignal::new(String::new());
    let last_input = RwSignal::new(Utc::now());

    let timediff = move || (Utc::now() - last_input.get()).num_seconds() > 30;

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

            if scan_input.is_empty() {
                return;
            }

            let money_args_clone = user_args.clone();

            spawn_local(async move {
                console_log(&format!("Input {scan_input}"));
                let article = get_article_by_barcode(scan_input.clone()).await;

                let article = match article {
                    Ok(value) => value,
                    Err(e) => {
                        error_signal.set(format!("Failed to fetch article from server: {e}"));
                        return;
                    }
                };

                match article {
                    None => {
                        console_log(&format!(
                            "No article could be found with barcode '{scan_input}'"
                        ));
                    }

                    Some(value) => {
                        console_log(&format!("Need to buy article: {}", value.name));
                        buy_article(
                            money_args_clone.user_id.get(),
                            value.id,
                            money_args_clone.money,
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
}
