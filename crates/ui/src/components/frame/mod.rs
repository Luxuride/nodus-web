//! The page frame: the rail, the main column, and whatever the route paints inside it.

use leptos::prelude::*;

use super::rail::{ExtraDoors, Rail};
#[component]
pub fn Frame(
    /// Whether the reader has a session. The rail is navigation, so it is only there for
    /// someone who can use it.
    #[prop(into)]
    signed_in: Signal<bool>,
    /// The address being shown.
    #[prop(into)]
    path: Signal<String>,
    /// The screen itself. A function, not a value, so a route change repaints the content
    /// without rebuilding the frame around it (which would drop the screen's state).
    inner: ChildrenFn,
    /// Doors only some readers have.
    extra: Option<ExtraDoors>,
) -> impl IntoView {
    let inner = StoredValue::new(inner);
    view! {
        <div class="min-h-screen bg-base-200 text-base-content">
            <Show when=move || signed_in.get()>
                <header class="sticky top-0 z-30 flex items-center justify-between gap-2 border-b border-base-300 bg-base-100 px-3 py-2 xl:hidden">
                    <a class="font-semibold tracking-tight" href="/">"Nodus"</a>
                </header>
            </Show>
            <div class="mx-auto flex w-full max-w-6xl flex-col gap-4 px-2 py-4 xl:flex-row">
                <Show when=move || signed_in.get()>
                    <Rail active=path extra=extra.clone()/>
                </Show>
                <main class="min-w-0 flex-1 pb-[calc(5.5rem+env(safe-area-inset-bottom))] xl:pb-0">
                    {move || inner.with_value(|inner| inner())}
                </main>
            </div>
        </div>
    }
}
