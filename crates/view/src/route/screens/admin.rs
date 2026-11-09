//! Staff routes: the page, its gate and its strip, with whatever the runtime holds.
//!
//! The section bodies arrive as the page's child, so the browser can put a bridge-backed
//! panel there and the stateless paint can put the same page's own unavailable note.

use std::sync::Arc;

use leptos::prelude::*;

use crate::Note;
use crate::chrome::StaffStanding;
use crate::pages::admin::AdminPage;
use crate::pages::moderation::ModerationPage;
use crate::route::Runtime;

#[component]
pub fn AdminScreen(runtime: Arc<dyn Runtime>, section: String) -> impl IntoView {
    let _ = &runtime;
    let standing = Signal::derive(|| None::<StaffStanding>);
    let missing: ChildrenFn =
        Arc::new(|| view! { <Note>"The staff tools need the app."</Note> }.into_any());
    view! {
        <AdminPage active=section standing=standing>{missing()}</AdminPage>
    }
}

#[component]
pub fn ModerationScreen(runtime: Arc<dyn Runtime>, section: String) -> impl IntoView {
    let _ = &runtime;
    let standing = Signal::derive(|| None::<StaffStanding>);
    let missing: ChildrenFn =
        Arc::new(|| view! { <Note>"The moderation tools need the app."</Note> }.into_any());
    view! {
        <ModerationPage active=section standing=standing>{missing()}</ModerationPage>
    }
}
