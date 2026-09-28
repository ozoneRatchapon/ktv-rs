use app::booth::{Booth, Placement, Requester};
use app::catalog::builtin_catalog;
use app::medley::{
    self, guess_span, nudge_span, Edge, Medley, MedleyBook, MedleyError, MedleySlot, PartSource, Span, GUESS_INTRO_SECS,
    GUESS_LEN_SECS, MAX_PARTS, MAX_SAVED, MIN_PART_SECS,
};
use app::types::{QueueItem, Song};

fn song(index: usize) -> Song {
    builtin_catalog()[index].clone()
}

fn sized(duration_secs: u32, intro_skip_secs: u32) -> Song {
    Song { duration_secs, intro_skip_secs, ..song(0) }
}

fn book_with(indices: &[usize]) -> MedleyBook {
    let mut book = MedleyBook::default();
    for &i in indices {
        book.add_song(&song(i)).unwrap();
    }
    book
}

#[test]
fn test_guess_starts_after_the_intro_and_runs_a_verse_and_chorus_inside_the_song() {
    assert_eq!(guess_span(&sized(240, 18)), Span { start: 18.0, end: 18.0 + GUESS_LEN_SECS });
    // No intro skip on the channel: still past the title card and intro music
    assert_eq!(guess_span(&sized(240, 0)), Span { start: GUESS_INTRO_SECS, end: GUESS_INTRO_SECS + GUESS_LEN_SECS });
    // Short song: cut at its end
    assert_eq!(guess_span(&sized(60, 18)), Span { start: 18.0, end: 60.0 });
    // Shorter than the intro: the whole song
    assert_eq!(guess_span(&sized(12, 18)), Span { start: 0.0, end: 12.0 });
    // Length unknown: no cap
    assert_eq!(guess_span(&sized(0, 0)).len(), GUESS_LEN_SECS);
}

#[test]
fn test_span_within_rejects_short_reversed_negative_and_past_the_end() {
    assert!(Span::within(10.0, 10.0 + MIN_PART_SECS, 240).is_some());
    for (start, end, duration) in [(10.0, 12.0, 240), (20.0, 10.0, 240), (-1.0, 30.0, 240), (200.0, 250.0, 240), (f64::NAN, 30.0, 240)] {
        assert_eq!(Span::within(start, end, duration), None, "{start}..{end} of {duration}");
    }
    assert!(Span::within(200.0, 250.0, 0).is_some(), "unknown length: no upper bound");
}

#[test]
fn test_nudge_keeps_the_part_inside_the_song_and_long_enough() {
    let span = Span { start: 10.0, end: 30.0 };
    assert_eq!(nudge_span(span, Edge::Start, -5.0, 240), Span { start: 5.0, end: 30.0 });
    assert_eq!(nudge_span(span, Edge::Start, -50.0, 240).start, 0.0);
    assert_eq!(nudge_span(span, Edge::Start, 50.0, 240).start, 30.0 - MIN_PART_SECS);
    assert_eq!(nudge_span(span, Edge::End, 500.0, 240).end, 240.0);
    assert_eq!(nudge_span(span, Edge::End, -50.0, 240).end, 10.0 + MIN_PART_SECS);
    assert_eq!(nudge_span(span, Edge::End, 500.0, 0).end, 530.0, "unknown length: no cap");
}

#[test]
fn test_a_marked_part_wins_over_a_guess_and_a_nudge_counts_as_marked() {
    let mut book = MedleyBook::default();
    let s = song(3);
    book.add_song(&s).unwrap();
    assert_eq!(book.draft.parts[0].source, PartSource::Guessed);
    assert_eq!(book.draft.parts[0].span, guess_span(&s));

    book.add_marked(&s, 40.0, 70.5).unwrap();
    book.add_song(&s).unwrap();
    assert_eq!(book.draft.parts[2].span, Span { start: 40.0, end: 70.5 }, "the part marked before is used");
    assert_eq!(book.draft.parts[2].source, PartSource::Marked);

    book.nudge(0, Edge::End, 5.0);
    assert_eq!(book.draft.parts[0].source, PartSource::Marked);
    assert_eq!(book.marked[&s.id], book.draft.parts[0].span, "the nudged part is remembered for the song");

    assert_eq!(book.add_marked(&s, 40.0, 41.0), Err(MedleyError::BadPart));
}

#[test]
fn test_draft_limits_order_and_removal() {
    let mut book = book_with(&[0, 1, 2]);
    book.move_up(2);
    book.move_down(0);
    let ids: Vec<&str> = book.draft.parts.iter().map(|p| p.song_id.as_str()).collect();
    assert_eq!(ids, [song(2).id.as_str(), song(0).id.as_str(), song(1).id.as_str()]);
    book.move_up(0);
    book.move_down(2);
    book.remove_part(9);
    assert_eq!(book.draft.parts.len(), 3, "out-of-range moves and removals do nothing");
    book.remove_part(1);
    assert_eq!(book.draft.parts.len(), 2);

    let mut full = MedleyBook::default();
    for i in 0..MAX_PARTS {
        full.add_song(&song(i % 5)).unwrap();
    }
    assert_eq!(full.add_song(&song(0)), Err(MedleyError::TooManyParts));
    // A marked part is still remembered when the draft is full
    assert_eq!(full.add_marked(&song(7), 30.0, 60.0), Err(MedleyError::TooManyParts));
    assert!(full.marked.contains_key(&song(7).id));
}

#[test]
fn test_save_needs_two_parts_names_it_and_replaces_by_title() {
    let mut book = book_with(&[0]);
    assert_eq!(book.save_draft(), Err(MedleyError::TooFewParts));
    book.add_song(&song(1)).unwrap();
    book.save_draft().unwrap();
    assert_eq!(book.saved[0].title, "Medley 1", "an untitled medley gets a name");

    book.edit(0);
    book.add_song(&song(2)).unwrap();
    book.save_draft().unwrap();
    assert_eq!(book.saved.len(), 1, "same title: the edit replaces it");
    assert_eq!(book.saved[0].parts.len(), 3);

    book.set_title(&"x".repeat(200));
    book.save_draft().unwrap();
    assert_eq!(book.saved.len(), 2);
    assert_eq!(book.saved[1].title.chars().count(), medley::MAX_TITLE_CHARS);
    book.delete(0);
    assert_eq!(book.saved.len(), 1);

    let mut full = book_with(&[0, 1]);
    for i in 0..MAX_SAVED {
        full.set_title(&format!("m{i}"));
        full.save_draft().unwrap();
    }
    full.set_title("one more");
    assert_eq!(full.save_draft(), Err(MedleyError::BookFull));
}

#[test]
fn test_slots_carry_title_position_and_span() {
    let book = book_with(&[0, 1, 2]);
    let medley = Medley { title: "  ".to_string(), ..book.draft.clone() };
    let slots = medley::slots(&medley).unwrap();
    assert_eq!(slots.len(), 3);
    assert_eq!(slots[1].0, song(1).id);
    assert_eq!(slots[1].1, MedleySlot { title: "Medley".to_string(), index: 1, count: 3, span: book.draft.parts[1].span });
    assert_eq!(slots[1].1.label(), "Medley 2/3 · Medley");
    assert!(!slots[0].1.is_join() && slots[2].1.is_join());
    assert_eq!(medley::slots(&Medley { parts: book.draft.parts[..1].to_vec(), ..medley }), Err(MedleyError::TooFewParts));
}

#[test]
fn test_sanitized_drops_bad_parts_and_short_medleys() {
    let mut book = book_with(&[0, 1]);
    book.save_draft().unwrap();
    book.saved[0].parts[1].span = Span { start: 50.0, end: 20.0 };
    book.draft.parts[0].span.end = f64::from(book.draft.parts[0].duration_secs) + 60.0;
    book.marked.insert("x".to_string(), Span { start: 5.0, end: 6.0 });
    let clean = book.sanitized();
    assert!(clean.saved.is_empty(), "a saved medley left with one part is dropped");
    assert_eq!(clean.draft.parts.len(), 1);
    assert!(!clean.marked.contains_key("x"));
}

#[test]
fn test_stored_book_round_trips_and_old_queue_items_load() {
    let book = book_with(&[0, 1]);
    let json = serde_json::to_string(&book).unwrap();
    assert_eq!(serde_json::from_str::<MedleyBook>(&json).unwrap(), book);
    // A queue item saved before medleys existed has no part
    let item = QueueItem { queue_id: 1, song: song(0), requester: "Guest".to_string(), part: None };
    let json = serde_json::to_string(&item).unwrap();
    assert!(!json.contains("part"), "no part key when there is none");
    assert_eq!(serde_json::from_str::<QueueItem>(&json).unwrap(), item);
}

fn medley_parts(indices: &[usize]) -> Vec<(Song, MedleySlot)> {
    let book = book_with(indices);
    let slots = medley::slots(&book.draft).unwrap();
    indices.iter().zip(slots).map(|(&i, (_, slot))| (song(i), slot)).collect()
}

fn stage(booth: &Booth) -> Vec<(String, Option<u32>)> {
    booth.current.iter().chain(&booth.queue).map(|it| (it.song.id.clone(), it.part.as_ref().map(|p| p.index))).collect()
}

#[test]
fn test_booth_adds_a_medley_as_back_to_back_parts() {
    let mut booth = Booth::default();
    assert!(booth.add_medley(medley_parts(&[0, 1, 2]), Requester::Singer, Placement::Back), "nothing playing: it starts");
    assert_eq!(
        stage(&booth),
        [(song(0).id, Some(0)), (song(1).id, Some(1)), (song(2).id, Some(2))]
    );
    assert_eq!(booth.current.as_ref().unwrap().start_at(true), booth.current.as_ref().unwrap().part.as_ref().unwrap().span.start);

    // Play now: the medley goes on stage, the queue waits after it
    let mut booth = Booth::default();
    booth.add(song(5), Requester::Singer, Placement::Now);
    booth.add(song(6), Requester::Guest, Placement::Back);
    assert!(booth.add_medley(medley_parts(&[0, 1]), Requester::Singer, Placement::Now));
    assert_eq!(stage(&booth), [(song(0).id, Some(0)), (song(1).id, Some(1)), (song(6).id, None)]);
    assert!(!booth.add_medley(Vec::new(), Requester::Singer, Placement::Now), "an empty medley adds nothing");
}

#[test]
fn test_next_never_splits_a_medley_being_sung() {
    let mut booth = Booth::default();
    booth.add_medley(medley_parts(&[0, 1, 2]), Requester::Singer, Placement::Now);
    booth.add(song(6), Requester::Guest, Placement::Back);
    booth.add(song(7), Requester::Tip, Placement::Next);
    assert_eq!(
        stage(&booth),
        [(song(0).id, Some(0)), (song(1).id, Some(1)), (song(2).id, Some(2)), (song(7).id, None), (song(6).id, None)],
        "the tip request plays after the medley, before the rest of the queue"
    );
    booth.add_medley(medley_parts(&[3, 4]), Requester::Singer, Placement::Next);
    assert_eq!(booth.queue[2].song.id, song(3).id, "a medley queued next also waits for this one");
    assert_eq!(booth.queue[3].song.id, song(4).id);
    // Last part on stage: next is next again
    booth.advance();
    booth.advance();
    booth.add(song(8), Requester::Priority, Placement::Next);
    assert_eq!(booth.queue[0].song.id, song(8).id);
}
