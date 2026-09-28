use app::catalog::builtin_catalog;
use app::links::{percent_decode, percent_encode};
use app::medley::{parse_fragment, share_fragment, Medley, MedleyBook, PartSource, Span, MAX_PARTS, MAX_TITLE_CHARS};
use app::types::Song;

fn find(code: &str) -> Option<Song> {
    builtin_catalog().iter().find(|s| s.code == code).cloned()
}

fn medley_of(indices: &[usize], title: &str) -> Medley {
    let mut book = MedleyBook::default();
    for &i in indices {
        book.add_song(&builtin_catalog()[i]).unwrap();
    }
    book.set_title(title);
    book.nudge(0, app::medley::Edge::Start, 2.5);
    book.draft
}

#[test]
fn test_percent_decode_undoes_encode_and_rejects_broken_escapes() {
    for text in ["", "abc", "เธอ & ฉัน = 100%", "a+b c,d:e"] {
        assert_eq!(percent_decode(&percent_encode(text)).as_deref(), Some(text));
    }
    assert_eq!(percent_decode("a+b").as_deref(), Some("a b"));
    for broken in ["%", "%4", "%zz", "%FF"] {
        assert_eq!(percent_decode(broken), None, "{broken}");
    }
}

#[test]
fn test_a_shared_medley_opens_as_it_was_built() {
    let medley = medley_of(&[0, 2, 6], "ยาวๆ & มันส์: 90s");
    let fragment = share_fragment(&medley);
    assert!(fragment.starts_with("#medley="));
    assert!(fragment.is_ascii(), "the fragment is percent-encoded: {fragment}");

    let opened = parse_fragment(&fragment).unwrap().open(find);
    assert_eq!(opened.skipped, 0);
    assert_eq!(opened.medley, medley, "titles, songs, times and marked / guessed all survive");
    assert_eq!(opened.medley.parts[0].source, PartSource::Marked);
    assert_eq!(opened.medley.parts[1].source, PartSource::Guessed);
}

#[test]
fn test_times_travel_to_a_tenth_of_a_second() {
    let mut medley = medley_of(&[0, 2], "");
    medley.parts[0].span = Span { start: 12.34, end: 60.0 };
    let fragment = share_fragment(&medley);
    assert!(fragment.contains(":12.3-60"), "{fragment}");
    assert_eq!(parse_fragment(&fragment).unwrap().parts[0].span, Span { start: 12.3, end: 60.0 });
}

#[test]
fn test_songs_missing_here_or_too_short_for_the_times_are_skipped_and_counted() {
    let code = &builtin_catalog()[0].code;
    let fragment = format!("#medley=x&parts={code}:10-40,99999:10-40,{code}:10-99999");
    let opened = parse_fragment(&fragment).unwrap().open(find);
    assert_eq!(opened.medley.parts.len(), 1);
    assert_eq!(opened.skipped, 2);
}

#[test]
fn test_unreadable_fragments_hold_no_medley_and_bad_parts_are_dropped() {
    for fragment in ["", "#", "#tip=1", "#medley=x", "#medley=x&parts=", "#parts=1:a-b", "#parts=12345:10-12"] {
        assert_eq!(parse_fragment(fragment), None, "{fragment}");
    }
    let shared = parse_fragment("parts=12345:10-40g,junk,:1-20,12345:-5-30").unwrap();
    assert_eq!(shared.title, "", "no title: the builder shows its stand-in");
    assert_eq!(shared.parts.len(), 1);
    assert_eq!(shared.parts[0].source, PartSource::Guessed);
}

#[test]
fn test_a_shared_medley_is_held_to_the_builders_limits() {
    let parts = vec!["12345:10-40"; MAX_PARTS + 5].join(",");
    let title = "ก".repeat(MAX_TITLE_CHARS * 2);
    let shared = parse_fragment(&format!("#medley={}&parts={parts}", percent_encode(&title))).unwrap();
    assert_eq!(shared.parts.len(), MAX_PARTS);
    assert_eq!(shared.title.chars().count(), MAX_TITLE_CHARS);
}
