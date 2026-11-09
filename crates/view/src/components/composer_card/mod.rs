//! The card a composer sits in: one border, one padding, one shadow, on both paints.

use leptos::prelude::*;
/// The card the composer lives in: one border, one padding, one shadow, so the server's
/// static box and the client's live one occupy the same pixels.
#[component]
pub fn ComposerCard(children: ChildrenFn) -> impl IntoView {
    let children = StoredValue::new(children);
    view! {
        <div class="card border-2 border-base-300 bg-base-100 shadow-sm has-[textarea:focus]:border-primary/50">
            <div class="card-body gap-2 p-3">{children.with_value(|children| children())}</div>
        </div>
    }
}
