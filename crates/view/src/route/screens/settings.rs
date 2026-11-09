//! Settings, drawn from the runtime's snapshot.
//!
//! Every card is the same card the browser mounts. What the runtime cannot answer yet is
//! drawn as the card's own unknown state instead of a second page builder.

use std::sync::Arc;

use leptos::prelude::*;
use nodus_instance::model::InstanceInfo;
use nodus_state::page::{PageEntry, PageItems, PageKind};

use crate::pages::settings::account::{AccountItem, AccountsCard, ModerationLinks, SessionCard};
use crate::pages::settings::feeds::{FeedItem, FeedsCard};
use crate::pages::settings::form::ProfileFields;
use crate::pages::settings::instance_card::InstanceCard;
use crate::pages::settings::lists::{FeaturedTagsCard, FilterItem, FiltersCard, TagItem};
use crate::pages::settings::nav::SettingsSection;
use crate::pages::settings::page::SettingsPage;
use crate::pages::settings::profile::{ClockCard, ClockStyle, SensitiveCard};
use crate::pages::settings::sync::SyncRows;
use crate::route::Runtime;

fn filters(entry: Option<PageEntry>) -> Signal<Vec<FilterItem>> {
    Signal::derive(move || match entry.as_ref().map(|entry| &entry.items) {
        Some(PageItems::Filters(items)) => items
            .iter()
            .map(|filter| FilterItem {
                id: filter.id.clone(),
                title: filter.title.clone(),
                detail: format!(
                    "{}: {}",
                    filter.filter_action.as_str(),
                    filter
                        .keywords
                        .iter()
                        .map(|keyword| keyword.keyword.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                ),
            })
            .collect(),
        _ => Vec::new(),
    })
}

fn tags(entry: Option<PageEntry>) -> Signal<Vec<TagItem>> {
    Signal::derive(move || match entry.as_ref().map(|entry| &entry.items) {
        Some(PageItems::FeaturedTags(items)) => items
            .iter()
            .map(|tag| TagItem {
                id: tag.id.clone(),
                label: format!("#{}", tag.name),
            })
            .collect(),
        _ => Vec::new(),
    })
}

#[component]
#[expect(
    clippy::too_many_lines,
    reason = "one section switch is clearer together"
)]
pub fn SettingsScreen(runtime: Arc<dyn Runtime>, section: SettingsSection) -> impl IntoView {
    let current = StoredValue::new(runtime.session());
    let host = Signal::derive(move || {
        current.with_value(|current| current.as_ref().map(|s| s.host.clone()).unwrap_or_default())
    });
    let acct = Signal::derive(move || {
        current.with_value(|current| current.as_ref().map(|s| s.acct.clone()).unwrap_or_default())
    });
    let accounts = Signal::derive(move || {
        current
            .with_value(|current| {
                current.as_ref().map(|session| AccountItem {
                    key: session.account_id.clone(),
                    label: session.acct.clone(),
                    host: session.host.clone(),
                    current: true,
                })
            })
            .into_iter()
            .collect()
    });
    let filter_entry = runtime.page(&PageKind::Filters);
    let tag_entry = runtime.page(&PageKind::FeaturedTags);
    let filter_items = filters(filter_entry);
    let tag_items = tags(tag_entry);
    let empty_add = StoredValue::new(Arc::new(|| ().into_any()) as ChildrenFn);
    let no_info = Signal::derive(|| None::<InstanceInfo>);
    let no_error = Signal::derive(|| None::<String>);
    let idle = Signal::derive(|| false);

    if let Some(body) = runtime.settings_body(section) {
        return view! {
            <SettingsPage section=Signal::derive(move || section) on_navigate=None>
                {let body = body.clone(); move || body()}
            </SettingsPage>
        }
        .into_any();
    }

    view! {
        <SettingsPage section=Signal::derive(move || section) on_navigate=None>
            {move || match section {
                SettingsSection::Account => view! {
                    <div class="flex flex-col gap-4">
                        <SessionCard host=host acct=acct on_sign_out=None/>
                        <AccountsCard accounts=accounts busy=idle error=no_error on_switch=None on_sign_out=None/>
                        <ModerationLinks on_mutes=None on_blocks=None/>
                        <ClockCard
                            selected=Signal::derive(|| ClockStyle::TwentyFour)
                            sample=Signal::derive(String::new)
                            on_select=None
                        />
                        <SensitiveCard automatic=Signal::derive(|| false) on_toggle=None/>
                        <FeedsCard feeds=Signal::derive(Vec::<FeedItem>::new) on_toggle=None/>
                    </div>
                }.into_any(),
                SettingsSection::Profile => view! {
                    <div class="flex flex-col gap-4">
                        <ProfileFields
                            display_name=Signal::derive(String::new)
                            note=Signal::derive(String::new)
                            max_note=500
                            busy=idle
                            message=no_error
                        />
                        <ClockCard
                            selected=Signal::derive(|| ClockStyle::TwentyFour)
                            sample=Signal::derive(String::new)
                            on_select=None
                        />
                        <SensitiveCard automatic=Signal::derive(|| false) on_toggle=None/>
                    </div>
                }.into_any(),
                SettingsSection::Nodus => {
                    let empty_add = empty_add.get_value();
                    view! {
                    <div class="flex flex-col gap-4">
                        <FeedsCard feeds=Signal::derive(Vec::<FeedItem>::new) on_toggle=None/>
                        <FeaturedTagsCard
                            tags=tag_items
                            busy=idle
                            error=no_error
                            on_delete=None
                            add_form=empty_add
                        />
                        <SyncRows
                            state=Signal::derive(|| None::<bool>)
                            busy=idle
                            result=no_error
                            on_ask=None
                        />
                    </div>
                }.into_any()
                },
                SettingsSection::Filters => {
                    let empty_add = empty_add.get_value();
                    view! {
                    <FiltersCard
                        filters=filter_items
                        busy=idle
                        error=no_error
                        on_delete=None
                        add_form=empty_add
                    />
                }.into_any()
                },
                SettingsSection::Instance => view! {
                    <InstanceCard info=no_info failed=Signal::derive(|| false)/>
                }.into_any(),
            }}
        </SettingsPage>
    }
    .into_any()
}
