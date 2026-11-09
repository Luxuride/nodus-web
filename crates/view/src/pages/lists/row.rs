//! One row of the picker: the shared list row, with the screen's edits left open.
//!
//! The box belongs to [`ListRow`](crate::lists::ListRow): the name, the address it selects
//! through, the replies policy and the border that marks the open list. What the shared row
//! cannot own is a list's two edits, because a rename swaps the title for an input and the
//! draft and busy signals behind it are the screen's — so the caller passes its form and its
//! delete button through `controls`, and a caller that has neither passes none and the row
//! draws exactly the same boxes without them.

use leptos::prelude::*;
use nodus_instance::model::List;

use crate::lists::list_row::{ListRow, ListRowProps};

/// One list in the picker, with whatever the screen hangs off it.
#[component]
pub fn ListPickerRow(
    /// The list this row shows.
    list: List,
    /// Whether the screen is reading this list right now.
    #[prop(optional)]
    selected: bool,
    /// Selects the list inside the app instead of following its address.
    #[prop(optional)]
    on_select: Option<Callback<String>>,
    /// The screen's own inline rename form and delete button.
    #[prop(optional)]
    controls: Option<ChildrenFn>,
) -> impl IntoView {
    // The shared row's slot is an optional prop, so it is filled from the props struct: the
    // builder's setter for a slot takes the slot itself, and a row that forwards an absent
    // one has nothing to hand it.
    ListRow(ListRowProps {
        list,
        selected,
        on_select,
        children: controls,
    })
}
