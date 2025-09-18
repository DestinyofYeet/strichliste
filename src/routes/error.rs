use leptos::prelude::*;
use reactive_stores::{Store, StoreField};
use thaw::{
    Button, ButtonAppearance, ConfigProvider, Dialog, DialogActions, DialogBody, DialogContent,
    DialogSurface, DialogTitle,
};

use crate::routes::state::{FrontendStore, FrontendStoreStoreFields};
#[component]
pub fn ErrorDisplay() -> impl IntoView {
    let store = expect_context::<Store<FrontendStore>>();
    let content = RwSignal::new("".to_string());
    let open = RwSignal::new(false);
    view! {
        {move || {
            match store.error().get().get() {
                Some(e) => {
                    open.set(true);
                    content.set(e);
                }
                None => open.set(false),
            }
        }}
        <ConfigProvider>
            <Dialog open>
                <DialogSurface>
                    <DialogBody>
                        <DialogTitle>"An error occured"</DialogTitle>
                        <DialogContent>{move || content.get()}</DialogContent>
                        <DialogActions>
                            <Button
                                appearance=ButtonAppearance::Primary
                                on_click=move |_| { store.error().writer().unwrap().set(None) }
                            >
                                "Ok"
                            </Button>
                        </DialogActions>
                    </DialogBody>
                </DialogSurface>
            </Dialog>
        </ConfigProvider>
    }
    .into_any()
}
