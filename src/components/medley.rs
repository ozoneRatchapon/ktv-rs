use dioxus::prelude::*;

use crate::booth::Placement;
use crate::browser;
use crate::components::practice::format_mark;
use crate::medley::{display_title, share_fragment, shown_title, Edge, Medley, MedleyBook, Opened, PartSource, SharedMedley, MIN_PARTS};

/// Seconds one nudge moves a part's start or end.
const NUDGE_SECS: f64 = 5.0;

/// Outcome of the last action, shown under the add field.
type Notice = Option<Result<String, String>>;

/// Queue tab → Medley: build a medley from song parts, save it, and play or queue it as back-to-back entries.
/// The book lives in context (`Signal<MedleyBook>`, provided by the app), so the practice row can add marked parts.
#[component]
pub fn MedleyPanel(
    /// Keypad code → add that song's part to the draft; the text to show.
    on_add_code: Callback<String, Result<String, String>>,
    /// Play now or queue a medley; the text to show.
    on_queue: Callback<(Medley, Placement), Result<String, String>>,
    /// A medley from the link this page was opened with, until opened or dismissed.
    mut shared: Signal<Option<SharedMedley>>,
    /// Look a shared medley's songs up in this songbook.
    on_open_shared: Callback<SharedMedley, Opened>,
) -> Element {
    let mut book = use_context::<Signal<MedleyBook>>();
    let mut code = use_signal(String::new);
    let mut notice = use_signal(|| None as Notice);
    // The last share link and the medley it holds: shown while the draft is unchanged, so it can be copied by
    // hand where the clipboard is blocked (plain http on a LAN address)
    let mut share_link = use_signal(|| None::<(Medley, String)>);
    let draft = book.read().draft.clone();
    let saved = book.read().saved.clone();
    let ready = draft.parts.len() >= MIN_PARTS;

    let mut add = move || {
        let typed = code();
        notice.set(Some(on_add_code.call(typed.trim().to_string())));
        if notice().is_some_and(|n| n.is_ok()) {
            code.set(String::new());
        }
    };
    let mut queue = move |medley: Medley, placement: Placement| notice.set(Some(on_queue.call((medley, placement))));
    let mut open_shared = move |link: SharedMedley| {
        let Opened { medley, skipped } = on_open_shared.call(link);
        if medley.parts.is_empty() {
            notice.set(Some(Err("None of its songs are in this songbook (yet): wait for the library to load, then open it again".to_string())));
            return;
        }
        let title = display_title(&medley).to_string();
        book.write().draft = medley;
        shared.set(None);
        notice.set(Some(Ok(match skipped {
            0 => format!("Opened: {title}"),
            1 => format!("Opened: {title} (1 part skipped: its song is not in this songbook)"),
            n => format!("Opened: {title} ({n} parts skipped: their songs are not in this songbook)"),
        })));
    };
    let mut share = move |medley: &Medley| {
        let link = format!("{}/{}", browser::page_origin().unwrap_or_default(), share_fragment(medley));
        share_link.set(Some((medley.clone(), link.clone())));
        spawn(async move {
            let copied = browser::copy_text(&link).await;
            let how = if copied { "Share link copied" } else { "Copy the share link below" };
            notice.set(Some(Ok(format!("{how}: whoever opens it gets this medley in their builder"))));
        });
    };

    rsx! {
        section { class: "medley-section", aria_label: "Medley",
            div { class: "queue-header-row",
                div { class: "queue-title-group",
                    h3 { class: "queue-heading", "MEDLEY" }
                    span { class: "queue-count-pill", "{draft.parts.len()} parts" }
                }
            }
            p { class: "medley-hint",
                "Sing parts of several songs back to back. Add a song by its code: its part is guessed (after the intro, about a verse and a chorus), so nudge it by ear. Or mark A and B while a song plays and press + Medley: a marked part is used whenever that song is added."
            }
            if let Some(link) = shared() {
                div { class: "medley-shared", role: "status",
                    span { "Shared with you: " }
                    span { class: "item-title", lang: "th", "{shown_title(&link.title)}" }
                    span { class: "queue-count-pill", "{link.parts.len()} parts" }
                    if !draft.parts.is_empty() {
                        span { class: "medley-hint", "Opening it replaces the medley being built." }
                    }
                    button { class: "antic-btn queue", onclick: move |_| open_shared(link.clone()), "Open" }
                    button { class: "antic-btn", onclick: move |_| shared.set(None), "Dismiss" }
                }
            }
            div { class: "medley-add",
                input {
                    id: "medley_code",
                    name: "medley_code",
                    class: "form-input",
                    r#type: "text",
                    inputmode: "numeric",
                    maxlength: "5",
                    placeholder: "5-digit song code",
                    aria_label: "Song code to add to the medley",
                    value: "{code}",
                    oninput: move |e| code.set(e.value()),
                    onkeydown: move |e| {
                        if e.key() == Key::Enter {
                            add();
                        }
                    },
                }
                button { class: "ctrl-btn practice-btn", onclick: move |_| add(), "Add" }
            }
            match notice() {
                Some(Ok(text)) => rsx! { p { class: "form-feedback", role: "status", "{text}" } },
                Some(Err(text)) => rsx! { p { class: "form-feedback error", role: "alert", "{text}" } },
                None => rsx! {},
            }

            ol { class: "medley-parts",
                for (i, part) in draft.parts.iter().enumerate() {
                    li { key: "{i}-{part.song_id}", class: "medley-part",
                        div { class: "medley-part-info",
                            span { class: "item-code", "#{part.code}" }
                            span { class: "item-title", lang: "th", "{part.title}" }
                            span { class: "medley-part-span",
                                "{format_mark(part.span.start)} → {format_mark(part.span.end)}"
                            }
                            match part.source {
                                PartSource::Guessed => rsx! {
                                    span { class: "medley-source guessed", title: "Guessed from the song's length: nudge it to the part you want", "guessed" }
                                },
                                PartSource::Marked => rsx! {
                                    span { class: "medley-source", title: "Marked by the host", "marked" }
                                },
                            }
                        }
                        div { class: "medley-part-actions",
                            for (edge, delta, label, title) in [
                                (Edge::Start, -NUDGE_SECS, "Start −5", "Start 5 s earlier"),
                                (Edge::Start, NUDGE_SECS, "Start +5", "Start 5 s later"),
                                (Edge::End, -NUDGE_SECS, "End −5", "End 5 s earlier"),
                                (Edge::End, NUDGE_SECS, "End +5", "End 5 s later"),
                            ] {
                                button {
                                    class: "item-order-btn",
                                    title,
                                    onclick: move |_| book.write().nudge(i, edge, delta),
                                    "{label}"
                                }
                            }
                            if i > 0 {
                                button { class: "item-order-btn", title: "Move up", onclick: move |_| book.write().move_up(i), "▲" }
                            }
                            if i + 1 < draft.parts.len() {
                                button { class: "item-order-btn", title: "Move down", onclick: move |_| book.write().move_down(i), "▼" }
                            }
                            button { class: "item-delete-btn", title: "Remove from the medley", onclick: move |_| book.write().remove_part(i), "✕" }
                        }
                    }
                }
            }

            if !draft.parts.is_empty() {
                div { class: "medley-actions",
                    input {
                        id: "medley_title",
                        name: "medley_title",
                        class: "form-input",
                        r#type: "text",
                        placeholder: "Medley title",
                        aria_label: "Medley title",
                        value: "{draft.title}",
                        oninput: move |e| book.write().set_title(&e.value()),
                    }
                    button {
                        class: "ctrl-btn action-btn primary-glow",
                        disabled: !ready,
                        title: "Play the medley now; the queue waits after it",
                        onclick: {
                            let medley = draft.clone();
                            move |_| queue(medley.clone(), Placement::Now)
                        },
                        "Play now"
                    }
                    button {
                        class: "ctrl-btn action-btn",
                        disabled: !ready,
                        title: "Add the medley to the end of the queue",
                        onclick: {
                            let medley = draft.clone();
                            move |_| queue(medley.clone(), Placement::Back)
                        },
                        "Queue"
                    }
                    button {
                        class: "ctrl-btn action-btn",
                        disabled: !ready,
                        title: "Keep this medley on this device (same title = replace)",
                        onclick: move |_| {
                            let result = book.write().save_draft();
                            let title = display_title(&book.peek().draft).to_string();
                            notice.set(Some(result.map(|()| format!("Saved: {title}")).map_err(|e| e.message())));
                        },
                        "Save"
                    }
                    button {
                        class: "ctrl-btn action-btn",
                        disabled: !ready,
                        title: "Copy a link that opens this medley in someone else's builder",
                        onclick: {
                            let medley = draft.clone();
                            move |_| share(&medley)
                        },
                        "Share"
                    }
                    button {
                        class: "ctrl-btn action-btn",
                        title: "Start a new medley",
                        onclick: move |_| book.write().clear_draft(),
                        "Clear"
                    }
                }
                if let Some((_, link)) = share_link().filter(|(shared, _)| *shared == draft) {
                    input {
                        id: "medley_share_link",
                        name: "medley_share_link",
                        class: "form-input medley-share-link",
                        r#type: "url",
                        readonly: true,
                        aria_label: "Share link for this medley",
                        value: "{link}",
                    }
                }
                if !ready {
                    p { class: "medley-hint", "Add at least {MIN_PARTS} parts to play or save it." }
                }
            }

            if !saved.is_empty() {
                h4 { class: "dj-heading", "SAVED MEDLEYS" }
                ul { class: "medley-saved",
                    for (i, medley) in saved.into_iter().enumerate() {
                        li { key: "{i}-{medley.title}", class: "medley-saved-item",
                            span { class: "item-title", lang: "th", "{display_title(&medley)}" }
                            span { class: "queue-count-pill", "{medley.parts.len()} parts" }
                            button {
                                class: "antic-btn queue",
                                onclick: {
                                    let medley = medley.clone();
                                    move |_| queue(medley.clone(), Placement::Back)
                                },
                                "Queue"
                            }
                            button { class: "antic-btn", onclick: move |_| book.write().edit(i), "Edit" }
                            button { class: "item-delete-btn", title: "Delete this medley", onclick: move |_| book.write().delete(i), "✕" }
                        }
                    }
                }
            }
        }
    }
}
