//! The settings row's shell.
//!
//! The select, the content-warning toggle and the field are the view's — the same boxes the
//! inline composer shows. Only the language picker is assembled here, because its list comes
//! from the instance and it owns a document listener; it arrives at the row as a slot, in the
//! place the view leaves for it.

use std::sync::Arc;

use leptos::prelude::*;
use nodus_client_core::model::Visibility;
use nodus_pages::pages::compose::form::options::{
    ComposerOptions as OptionsRow, ComposerOptionsProps as OptionsRowProps,
};

/// The options row and the content-warning field.
#[component]
pub fn ComposerOptions(
    visibility: ReadSignal<Visibility>,
    set_visibility: WriteSignal<Visibility>,
    warning_open: ReadSignal<bool>,
    set_warning_open: WriteSignal<bool>,
    sensitive: ReadSignal<bool>,
    set_sensitive: WriteSignal<bool>,
    spoiler: ReadSignal<String>,
    set_spoiler: WriteSignal<String>,
    language: ReadSignal<String>,
    set_language: WriteSignal<String>,
) -> impl IntoView {
    // A slot rather than a child: the row places the picker between the visibility select and
    // the warning toggle, so it cannot be the trailing `children` block.
    let picker = Arc::new(move || {
        view! { <crate::ui::language::LanguagePicker language=language set_language=set_language/> }
            .into_any()
    }) as ChildrenFn;

    // The props struct is filled directly because view syntax can hand a component a slot but
    // not the absence of one.
    OptionsRow(OptionsRowProps {
        visibility: visibility.into(),
        warning_open: warning_open.into(),
        sensitive: sensitive.into(),
        spoiler: spoiler.into(),
        on_visibility: Some(Callback::new(move |chosen: Visibility| {
            set_visibility.set(chosen)
        })),
        on_toggle_warning: Some(Callback::new(move |()| {
            set_warning_open.update(|open| *open = !*open);
        })),
        on_sensitive: Some(Callback::new(move |flag: bool| set_sensitive.set(flag))),
        on_spoiler: Some(Callback::new(move |text: String| set_spoiler.set(text))),
        language: Some(picker),
    })
}
