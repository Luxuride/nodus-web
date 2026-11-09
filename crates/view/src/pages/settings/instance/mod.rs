//! The instance behind this account: what it calls itself, what it asks of you, who runs it.
//!
//! Nothing here reads `/api/v2/instance`: the caller does that and hands the answer down as
//! a signal, because this crate has no transport.

use leptos::prelude::*;
use nodus_instance::model::InstanceInfo;

/// The name, the version, the banner and the facts a joiner asks about.
#[component]
pub fn Identity(
    /// What the server said about itself, once it has answered.
    #[prop(into)]
    info: Signal<Option<InstanceInfo>>,
) -> impl IntoView {
    let thumbnail = move || {
        info.get()
            .and_then(|info| info.thumbnail)
            .map_or_else(String::new, |thumbnail| thumbnail.url)
    };
    let title = move || info.get().map_or_else(String::new, |info| info.title);
    let line = move || {
        info.get().map_or_else(String::new, |info| {
            format!("{} · v{}", info.domain, info.version)
        })
    };
    let summary = move || {
        info.get()
            .map_or_else(String::new, |info| info.short_description)
    };
    let registrations = move || {
        info.get().and_then(|info| info.registrations).map_or_else(
            || "not stated".to_owned(),
            |registrations| {
                if !registrations.enabled {
                    "closed".to_owned()
                } else if registrations.approval_required {
                    "open, with approval".to_owned()
                } else {
                    "open".to_owned()
                }
            },
        )
    };
    let active = move || {
        info.get()
            .and_then(|info| info.usage)
            .and_then(|usage| usage.users)
            .map_or_else(
                || "not stated".to_owned(),
                |users| users.active_month.to_string(),
            )
    };
    let contact = move || {
        info.get()
            .and_then(|info| info.contact)
            .map_or_else(String::new, |contact| contact.email)
    };
    let software = move || info.get().map_or_else(String::new, |info| info.source_url);

    view! {
        <div class="flex items-start gap-3">
            <Show when=move || info.get().is_some_and(|info| info.thumbnail.is_some())>
                <img class="h-16 w-16 rounded-box object-cover" width="64" height="64" decoding="async" alt="" src=thumbnail/>
            </Show>
            <div class="min-w-0">
                <p class="truncate font-medium">{title}</p>
                <p class="truncate text-xs opacity-60">{line}</p>
                <p class="mt-1 text-sm opacity-80">{summary}</p>
            </div>
        </div>

        <dl class="grid grid-cols-[9rem_minmax(0,1fr)] gap-x-3 gap-y-1 text-sm">
            <dt class="opacity-60">"Registrations"</dt>
            <dd>{registrations}</dd>
            <dt class="opacity-60">"Active this month"</dt>
            <dd class="tabular-nums">{active}</dd>
            <dt class="opacity-60">"Contact"</dt>
            <dd class="truncate">{contact}</dd>
            <dt class="opacity-60">"Software"</dt>
            <dd class="truncate">{software}</dd>
        </dl>
    }
}

/// The rules, or a plain statement that the server publishes none.
#[component]
pub fn Rules(
    /// What the server said about itself, once it has answered.
    #[prop(into)]
    info: Signal<Option<InstanceInfo>>,
) -> impl IntoView {
    let rules = move || info.get().map_or_else(Vec::new, |info| info.rules);
    let empty = move || rules().is_empty();

    view! {
        <Show when=move || !empty()>
            <div>
                <h3 class="mb-1 text-sm font-medium">"Rules"</h3>
                <ol class="list-decimal pl-5 text-sm">
                    <For each=rules key=|rule| rule.id.clone() let:rule>
                        <li class="py-0.5">
                            <span>{rule.text.clone()}</span>
                            {rule
                                .hint
                                .clone()
                                .map(|hint| view! { <span class="block text-xs opacity-60">{hint}</span> })}
                        </li>
                    </For>
                </ol>
            </div>
        </Show>

        <Show when=empty>
            <p class="text-sm opacity-60">"This instance publishes no rules."</p>
        </Show>
    }
}
