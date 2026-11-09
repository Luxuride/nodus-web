//! The Settings page shell: its title, its tab strip and the slot the cards fill.

use leptos::prelude::*;

use crate::components::screen::Screen;

use super::nav::{SettingsSection, SettingsTabs};

/// The page around the settings cards.
#[component]
pub fn SettingsPage(
    #[prop(into)] section: Signal<SettingsSection>,
    #[prop(optional_no_strip)] on_navigate: Option<Callback<SettingsSection>>,
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <Screen>
            <h1 class="mb-5 text-xl font-semibold tracking-tight">"Settings"</h1>
            <SettingsTabs section=section on_navigate=on_navigate/>
            {children()}
        </Screen>
    }
}
