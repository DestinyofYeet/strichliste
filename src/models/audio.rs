use leptos::{
    leptos_dom::logging::console_log,
    prelude::{expect_context, GetUntracked, NodeRef},
    task::spawn_local,
};
use reactive_stores::{Store, StoreField};
use serde::{Deserialize, Serialize};

use crate::{
    models::Money,
    routes::{
        state::{set_error, FrontendStore, FrontendStoreStoreFields},
        user::get_item_sound_url,
    },
};

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum AudioPlayback {
    Failed,
    Undo,
    Deposit(Money),
    Sent(Money),
    Withdraw(Money),
    Bought(i64),
}

/*
   How to use a callback

   let error_callback = Closure::once(Box::new(move |_: web_sys::Event| {
       // Error playing audio, fall back to TTS
       tts_play(&tts_text);
    }));

    // Attempt to add the error event listener
    let _ = audio.add_event_listener_with_callback("error", error_callback.as_ref().unchecked_ref());
*/

pub fn play_sound(audio_playback: AudioPlayback) {
    use leptos::web_sys::{js_sys, Url};
    spawn_local(async move {
        let store = expect_context::<Store<FrontendStore>>();
        let audio_ref: NodeRef<leptos::html::Audio> = expect_context();
        let audio = match audio_ref.get_untracked() {
            Some(val) => val,
            None => {
                console_log("Failed to get audio node");
                return;
            }
        };

        let mut cached_sounds = store.cached_sounds().writer().unwrap();

        let url = match cached_sounds.get(&audio_playback) {
            Some(value) => value,
            None => {
                let sound = match get_item_sound_url(audio_playback).await {
                    Ok(value) => value,
                    Err(e) => {
                        set_error(format!("Failed to fetch sound: {e}"));
                        return;
                    }
                };

                let sound: &[u8] = &sound;

                let blob = leptos::web_sys::Blob::new_with_u8_array_sequence(&js_sys::Array::of1(
                    &js_sys::Uint8Array::from(sound),
                ))
                .unwrap();

                let url = Url::create_object_url_with_blob(&blob).unwrap();

                _ = cached_sounds.insert(audio_playback, url);

                cached_sounds.get(&audio_playback).unwrap()
            }
        };

        audio.set_src(url);
        match audio.play() {
            Ok(_) => {}
            Err(e) => console_log(&format!("Failed to play audio: {e:#?}")),
        }
    });
}
