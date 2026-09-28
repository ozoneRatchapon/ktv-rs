use app::booth::{Booth, Placement, Requester};
use app::catalog::builtin_catalog;
use app::medley::{slots, MedleyBook};
use app::score::{medley_total, record, MedleyTake, TakeResult, TuningSummary, PERFECT_CENTS, RANDOM_CENTS};

/// Cents off that give `score` (the tuning scale is linear between perfect and random).
fn cents_for(score: u8) -> f32 {
    RANDOM_CENTS - f32::from(score) / 100.0 * (RANDOM_CENTS - PERFECT_CENTS)
}

fn part_take(id: u64, index: u32, count: u32, score: u8, voice: Option<u8>) -> TakeResult {
    let summary = TuningSummary { notes: 20, mean_abs_cents: Some(cents_for(score)) };
    TakeResult::new("song", "title", "artist", summary, f64::from(index))
        .unwrap()
        .with_part(voice)
        .with_medley(Some(MedleyTake { id, index, count, title: "Mix".to_string() }))
}

fn sing(history: &mut Vec<TakeResult>, takes: impl IntoIterator<Item = TakeResult>) {
    takes.into_iter().for_each(|t| record(history, t));
}

#[test]
fn test_the_last_part_brings_the_medley_total_earlier_parts_do_not() {
    let mut history = Vec::new();
    sing(&mut history, [part_take(7, 0, 3, 80, None), part_take(7, 1, 3, 90, None)]);
    assert_eq!(medley_total(&history, &history[0]), None, "part 2 of 3 is not the end");
    let last = part_take(7, 2, 3, 70, None);
    let total = medley_total(&history, &last).unwrap();
    assert_eq!((total.score, total.scored, total.count, total.title.as_str()), (80, 3, 3, "Mix"));
    // Same result once the last take is in the history too (it is recorded before the card shows)
    record(&mut history, last.clone());
    assert_eq!(medley_total(&history, &last), Some(total));
}

#[test]
fn test_a_replayed_part_counts_once_with_its_latest_take() {
    let mut history = Vec::new();
    sing(&mut history, [part_take(7, 0, 2, 20, None), part_take(7, 0, 2, 60, None)]);
    let total = medley_total(&history, &part_take(7, 1, 2, 100, None)).unwrap();
    assert_eq!((total.score, total.scored), (80, 2));
}

#[test]
fn test_other_medleys_other_voices_and_unscored_parts_stay_out() {
    let mut history = Vec::new();
    let unscored = TakeResult::new("s", "t", "a", TuningSummary { notes: 1, mean_abs_cents: Some(5.0) }, 0.0)
        .unwrap()
        .with_medley(Some(MedleyTake { id: 7, index: 1, count: 3, title: "Mix".to_string() }));
    sing(&mut history, [part_take(3, 0, 3, 10, None), part_take(7, 0, 3, 90, Some(2)), part_take(7, 0, 3, 50, None), unscored]);
    let total = medley_total(&history, &part_take(7, 2, 3, 70, None)).unwrap();
    assert_eq!((total.score, total.scored, total.count), (60, 2, 3), "voice 1 only, medley 7 only, part 2 had no score");
    let duet = medley_total(&history, &part_take(7, 2, 3, 70, Some(2))).unwrap();
    assert_eq!((duet.score, duet.scored), (80, 2), "singer 2's own total");
}

#[test]
fn test_a_whole_song_or_an_all_silent_medley_has_no_total() {
    let song = TakeResult::new("s", "t", "a", TuningSummary { notes: 20, mean_abs_cents: Some(10.0) }, 0.0).unwrap();
    assert_eq!(medley_total(&[], &song), None);
    let quiet = TakeResult::new("s", "t", "a", TuningSummary { notes: 1, mean_abs_cents: Some(5.0) }, 0.0)
        .unwrap()
        .with_medley(Some(MedleyTake { id: 1, index: 0, count: 1, title: String::new() }));
    assert_eq!(medley_total(&[], &quiet), None);
}

#[test]
fn test_every_part_of_one_queued_medley_shares_its_take_id_and_the_next_medley_does_not() {
    let mut book = MedleyBook::default();
    for i in [0, 2, 6] {
        book.add_song(&builtin_catalog()[i]).unwrap();
    }
    let parts = |book: &MedleyBook| {
        slots(&book.draft).unwrap().into_iter().map(|(_, slot)| (builtin_catalog()[0].clone(), slot)).collect::<Vec<_>>()
    };
    let mut booth = Booth::default();
    booth.add_medley(parts(&book), Requester::Singer, Placement::Now);
    booth.add_medley(parts(&book), Requester::Singer, Placement::Back);
    let takes: Vec<MedleyTake> =
        booth.current.iter().chain(&booth.queue).map(|it| it.medley_take().unwrap()).collect();
    assert_eq!(takes.iter().map(|t| (t.id, t.index)).collect::<Vec<_>>()[..3], [(takes[0].id, 0), (takes[0].id, 1), (takes[0].id, 2)]);
    assert!(takes[3..].iter().all(|t| t.id == takes[3].id && t.id != takes[0].id));
}
