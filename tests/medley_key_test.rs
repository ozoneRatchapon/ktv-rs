use app::catalog::builtin_catalog;
use app::chords::{order_cost, smoothest_order, Chord, ChordChart, ChordCharts, Key, KeyStep};
use app::medley::{key_order, part_keys, MedleyBook};

fn key(tonic: u8, minor: bool) -> Key {
    Key { tonic, minor }
}

/// A chart with these chords every 4 s from `from`.
fn chart(from: f64, chords: &[&str]) -> ChordChart {
    let mut chart = ChordChart::default();
    for (i, name) in chords.iter().enumerate() {
        chart.insert(from + 4.0 * i as f64, name.parse::<Chord>().unwrap());
    }
    chart
}

#[test]
fn test_key_from_common_progressions() {
    assert_eq!(chart(0.0, &["C", "G", "Am", "F", "C", "G", "F", "C"]).key_between(0.0, 40.0), Some(key(0, false)));
    assert_eq!(chart(0.0, &["G", "D", "Em", "C", "G", "D", "G"]).key_between(0.0, 40.0), Some(key(7, false)));
    assert_eq!(chart(0.0, &["Am", "Dm", "E7", "Am", "Dm", "E", "Am"]).key_between(0.0, 40.0), Some(key(9, true)));
    assert_eq!(chart(0.0, &["Bb", "F/A", "Gm", "Eb", "F", "Bb"]).key_between(0.0, 40.0), Some(key(10, false)));
}

#[test]
fn test_key_uses_only_the_chords_sounding_in_the_part() {
    // Verse in C, then a modulation to E from 32 s
    let mut c = chart(0.0, &["C", "F", "G", "C", "Am", "F", "G", "C"]);
    for (i, name) in ["E", "A", "B7", "E", "C#m", "A", "B", "E"].iter().enumerate() {
        c.insert(32.0 + 4.0 * i as f64, name.parse().unwrap());
    }
    assert_eq!(c.key_between(0.0, 32.0), Some(key(0, false)));
    assert_eq!(c.key_between(32.0, 64.0), Some(key(4, false)));
    // A part starting mid-chord counts the chord already sounding
    assert_eq!(c.key_between(33.0, 45.0), Some(key(4, false)));
    // Too few chords to say
    assert_eq!(c.key_between(0.0, 7.0), None);
    assert_eq!(ChordChart::default().key_between(0.0, 60.0), None);
}

#[test]
fn test_key_names_and_steps_on_the_circle_of_fifths() {
    assert_eq!(key(0, false).to_string(), "C");
    assert_eq!(key(9, true).to_string(), "Am");
    assert_eq!(key(10, false).to_string(), "Bb");
    assert_eq!(key(1, true).to_string(), "C#m");
    let c = key(0, false);
    assert_eq!(c.step(c), KeyStep::Same);
    assert_eq!(c.step(key(7, false)), KeyStep::Near); // G
    assert_eq!(c.step(key(5, false)), KeyStep::Near); // F
    assert_eq!(c.step(key(9, true)), KeyStep::Near); // relative minor
    assert_eq!(c.step(key(4, true)), KeyStep::Near); // Em = relative of G
    assert_eq!(c.step(key(2, false)), KeyStep::Far); // D
    assert_eq!(c.step(key(6, false)), KeyStep::Far); // F#
    assert_eq!(c.fifths_apart(key(6, false)), 6);
    assert_eq!(KeyStep::Far.label(), "far key");
}

#[test]
fn test_smoothest_order_walks_the_circle() {
    // C, E, G, D, A: best walk is C G D A E (or its reverse)
    let keys = [key(0, false), key(4, false), key(7, false), key(2, false), key(9, false)];
    let order = smoothest_order(&keys);
    let walked = order.iter().map(|&i| keys[i]).collect::<Vec<_>>();
    assert_eq!(order_cost(&walked), 8);
    assert_eq!(order, [0, 2, 3, 4, 1]);
    // Two parts: nothing to reorder
    assert_eq!(smoothest_order(&keys[..2]), [0, 1]);
}

#[test]
fn test_key_order_only_when_every_key_is_known_and_it_helps() {
    let (c, g, d) = (Some(key(0, false)), Some(key(7, false)), Some(key(2, false)));
    assert_eq!(key_order(&[c, d, g]), Some(vec![0, 2, 1]));
    assert_eq!(key_order(&[c, g, d]), None); // already smooth
    assert_eq!(key_order(&[c, None, g]), None);
}

#[test]
fn test_part_keys_read_each_songs_chart_and_reorder_applies() {
    let songs = builtin_catalog();
    let mut book = MedleyBook::default();
    for song in &songs[..3] {
        book.add_song(song).unwrap();
    }
    let spans = book.draft.parts.iter().map(|p| p.span).collect::<Vec<_>>();
    let mut charts = ChordCharts::new();
    charts.insert(songs[0].id.clone(), chart(spans[0].start, &["C", "F", "G", "C"]));
    charts.insert(songs[2].id.clone(), chart(spans[2].start, &["G", "C", "D", "G"]));
    // Chords outside the part say nothing about it
    charts.insert(songs[1].id.clone(), chart(spans[1].end + 1.0, &["D", "G", "A", "D"]));
    assert_eq!(part_keys(&book.draft, &charts), [Some(key(0, false)), None, Some(key(7, false))]);

    let ids = |b: &MedleyBook| b.draft.parts.iter().map(|p| p.song_id.clone()).collect::<Vec<_>>();
    let before = ids(&book);
    book.reorder(&[2, 0, 1]);
    assert_eq!(ids(&book), [before[2].clone(), before[0].clone(), before[1].clone()]);
    // Not a full reordering: ignored
    for bad in [&[0, 1][..], &[0, 0, 1], &[0, 1, 3]] {
        book.reorder(bad);
        assert_eq!(ids(&book), [before[2].clone(), before[0].clone(), before[1].clone()]);
    }
}
