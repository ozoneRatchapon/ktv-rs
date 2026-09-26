use app::booth::{Booth, Placement, Requester, KEY_RANGE};
use app::catalog::builtin_catalog;
use app::types::GuideTrack;

fn song(index: usize) -> app::types::Song {
    builtin_catalog()[index].clone()
}

fn ids(booth: &Booth) -> Vec<String> {
    booth.queue.iter().map(|it| it.song.id.clone()).collect()
}

#[test]
fn test_first_request_goes_on_stage_whatever_the_placement() {
    for placement in [Placement::Now, Placement::Next, Placement::Back] {
        let mut booth = Booth::default();
        assert!(booth.add(song(0), Requester::Guest, placement), "{placement:?}");
        assert_eq!(booth.current.as_ref().map(|c| c.song.id.as_str()), Some(song(0).id.as_str()));
        assert!(booth.queue.is_empty());
    }
}

#[test]
fn test_placement_while_a_song_plays() {
    let mut booth = Booth::default();
    booth.add(song(0), Requester::Singer, Placement::Now);
    assert!(!booth.add(song(1), Requester::Guest, Placement::Back));
    assert!(!booth.add(song(2), Requester::Priority, Placement::Next));
    assert_eq!(ids(&booth), [song(2).id, song(1).id]);
    // Play now replaces the current song and leaves the queue alone
    assert!(booth.add(song(3), Requester::Keypad, Placement::Now));
    assert_eq!(booth.current.as_ref().unwrap().song.id, song(3).id);
    assert_eq!(booth.queue.len(), 2);
}

#[test]
fn test_queue_ids_are_unique_and_requester_is_labelled() {
    let mut booth = Booth { next_queue_id: 7, ..Booth::default() };
    booth.add(song(0), Requester::AddUrl, Placement::Back);
    booth.add(song(1), Requester::AutoDj, Placement::Back);
    let curr = booth.current.as_ref().unwrap();
    assert_eq!((curr.queue_id, curr.requester.as_str()), (7, "YouTube Direct"));
    assert_eq!((booth.queue[0].queue_id, booth.queue[0].requester.as_str()), (8, "Smart Auto-DJ"));
    assert_eq!(booth.next_queue_id, 9);
}

#[test]
fn test_advance_takes_the_head_then_empties_the_stage() {
    let mut booth = Booth::default();
    for i in 0..3 {
        booth.add(song(i), Requester::Guest, Placement::Back);
    }
    assert!(booth.advance());
    assert_eq!(booth.current.as_ref().unwrap().song.id, song(1).id);
    assert!(booth.advance());
    assert!(!booth.advance());
    assert!(booth.current.is_none());
}

#[test]
fn test_key_shifts_clamp_to_range() {
    let mut booth = Booth::default();
    booth.add(song(0), Requester::Guest, Placement::Back);
    booth.add(song(1), Requester::Guest, Placement::Back);
    booth.shift_current_key(100);
    assert_eq!(booth.current.as_ref().unwrap().key_shift, *KEY_RANGE.end());
    booth.reset_current_key();
    assert_eq!(booth.current.as_ref().unwrap().key_shift, 0);
    let qid = booth.queue[0].queue_id;
    booth.shift_item_key(qid, -100);
    assert_eq!(booth.queue[0].key_shift, *KEY_RANGE.start());
    booth.shift_item_key(9999, 1); // unknown id: no-op
}

#[test]
fn test_reorder_remove_and_clear_ignore_out_of_range() {
    let mut booth = Booth::default();
    for i in 0..4 {
        booth.add(song(i), Requester::Guest, Placement::Back);
    }
    booth.move_up(0);
    booth.move_down(2);
    assert_eq!(ids(&booth), [song(1).id, song(2).id, song(3).id], "edges are no-ops");
    booth.move_down(0);
    booth.move_up(2);
    assert_eq!(ids(&booth), [song(2).id, song(3).id, song(1).id]);
    let qid = booth.queue[1].queue_id;
    booth.remove(qid);
    assert_eq!(ids(&booth), [song(2).id, song(1).id]);
    booth.clear_queue();
    assert!(booth.queue.is_empty());
    assert!(booth.current.is_some(), "clearing the queue keeps the current song");
}

#[test]
fn test_set_guide_updates_every_copy() {
    let mut booth = Booth::default();
    booth.add(song(0), Requester::Guest, Placement::Back);
    booth.add(song(0), Requester::Guest, Placement::Back);
    booth.add(song(1), Requester::Guest, Placement::Back);
    let guide = GuideTrack { video_id: "qdaeYIGnpiA".to_string(), offset_secs: 2.5, rate: 1.0 };
    booth.set_guide(&song(0).id, Some(&guide));
    assert_eq!(booth.current.as_ref().unwrap().song.guide.as_ref(), Some(&guide));
    assert_eq!(booth.queue[0].song.guide.as_ref(), Some(&guide));
    assert_eq!(booth.queue[1].song.guide, song(1).guide, "other songs untouched");
    booth.set_guide(&song(0).id, None);
    assert!(booth.queue[0].song.guide.is_none());
}
