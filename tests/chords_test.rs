use app::chords::{Chord, ChordChart, ChordParseError, MERGE_SECS};

fn chord(text: &str) -> Chord {
    text.parse().unwrap_or_else(|e| panic!("{text:?}: {e}"))
}

#[test]
fn test_parse_and_print_canonical_names() {
    for (typed, shown) in [
        ("C", "C"),
        ("Am", "Am"),
        ("am", "Am"),
        ("F#m7", "F#m7"),
        ("Bbmaj7", "Bbmaj7"),
        ("BbM7", "Bbmaj7"),
        ("C♯m", "C#m"),
        ("E♭", "Eb"),
        ("Dsus", "Dsus4"),
        ("Gadd2", "Gadd9"),
        ("Bm7b5", "Bm7b5"),
        ("Bdim", "Bdim"),
        ("B°", "Bdim"),
        ("C+", "Caug"),
        ("Amin7", "Am7"),
        ("G/B", "G/B"),
        ("D/F#", "D/F#"),
        (" Em ", "Em"),
        ("A5", "A5"),
    ] {
        assert_eq!(chord(typed).to_string(), shown, "{typed:?}");
    }
}

#[test]
fn test_parse_rejects_non_chords_with_a_reason() {
    assert_eq!("".parse::<Chord>(), Err(ChordParseError::Empty));
    assert_eq!("   ".parse::<Chord>(), Err(ChordParseError::Empty));
    assert_eq!("Hm".parse::<Chord>(), Err(ChordParseError::Root));
    assert_eq!("ลา".parse::<Chord>(), Err(ChordParseError::Root));
    assert_eq!("Cxyz".parse::<Chord>(), Err(ChordParseError::Quality));
    assert_eq!("Am7 ".parse::<Chord>().map(|c| c.to_string()), Ok("Am7".to_string()), "trailing space trimmed");
    assert_eq!("G/".parse::<Chord>(), Err(ChordParseError::Bass));
    assert_eq!("G/X".parse::<Chord>(), Err(ChordParseError::Bass));
    assert_eq!("G/Bm".parse::<Chord>(), Err(ChordParseError::Bass));
    assert!(!ChordParseError::Quality.to_string().is_empty());
}

#[test]
fn test_transpose_keeps_quality_bass_and_spelling() {
    assert_eq!(chord("Am").transposed(2).to_string(), "Bm");
    assert_eq!(chord("G/B").transposed(1).to_string(), "G#/C");
    assert_eq!(chord("Bb").transposed(1).to_string(), "B");
    assert_eq!(chord("Eb").transposed(-1).to_string(), "D");
    assert_eq!(chord("Eb").transposed(1).to_string(), "E");
    assert_eq!(chord("Ab").transposed(-12).to_string(), "Ab");
    assert_eq!(chord("C").transposed(-1).to_string(), "B");
    assert_eq!(chord("Dbmaj7").transposed(1).to_string(), "Dmaj7");
    assert_eq!(chord("Dbmaj7").transposed(2).to_string(), "Ebmaj7", "flat spelling kept");
    assert_eq!(chord("C#m").transposed(2).to_string(), "D#m", "sharp spelling kept");
}

#[test]
fn test_chart_stays_sorted_and_merges_retaps() {
    let mut chart = ChordChart::default();
    chart.insert(10.0, chord("G"));
    chart.insert(2.0, chord("C"));
    chart.insert(6.0, chord("Am"));
    let names = |c: &ChordChart| c.marks().iter().map(|m| (m.at_secs, m.chord.clone())).collect::<Vec<_>>();
    assert_eq!(names(&chart), [(2.0, "C".into()), (6.0, "Am".into()), (10.0, "G".into())]);
    chart.insert(6.0 + MERGE_SECS / 2.0, chord("Em"));
    assert_eq!(chart.marks().len(), 3, "a re-tap replaces");
    assert_eq!(chart.marks()[1].chord, "Em");
    chart.insert(-3.0, chord("D"));
    assert_eq!(chart.marks()[0].at_secs, 0.0, "clamped to the start");
    chart.remove(0);
    chart.remove(99);
    assert_eq!(chart.marks().len(), 3);
}

#[test]
fn test_position_current_and_next() {
    let mut chart = ChordChart::default();
    assert_eq!(chart.position(5.0), (None, None));
    chart.insert(2.0, chord("C"));
    chart.insert(6.0, chord("G"));
    assert_eq!(chart.position(0.0), (None, Some(0)), "before the first change");
    assert_eq!(chart.position(2.0), (Some(0), Some(1)), "a change starts at its mark");
    assert_eq!(chart.position(5.9), (Some(0), Some(1)));
    assert_eq!(chart.position(6.0), (Some(1), None));
    assert_eq!(chart.position(600.0), (Some(1), None));
}

#[test]
fn test_palette_is_first_use_order() {
    let mut chart = ChordChart::default();
    for (t, c) in [(1.0, "C"), (2.0, "G"), (3.0, "Am"), (4.0, "G"), (5.0, "C"), (6.0, "F")] {
        chart.insert(t, chord(c));
    }
    assert_eq!(chart.palette(), ["C", "G", "Am", "F"]);
}

#[test]
fn test_stored_charts_round_trip_and_bad_marks_are_dropped() {
    let mut chart = ChordChart::default();
    chart.insert(1.5, chord("Am"));
    chart.insert(4.0, chord("F"));
    let json = chart.to_json();
    assert_eq!(json, r#"[{"at_secs":1.5,"chord":"Am"},{"at_secs":4.0,"chord":"F"}]"#);
    assert_eq!(serde_json::from_str::<ChordChart>(&json).unwrap(), chart);
    let edited: ChordChart =
        serde_json::from_str(r#"[{"at_secs":9,"chord":"G"},{"at_secs":1,"chord":"nope"},{"at_secs":3,"chord":"bb"}]"#).unwrap();
    let clean = edited.sanitized();
    assert_eq!(clean.marks().iter().map(|m| m.chord.as_str()).collect::<Vec<_>>(), ["Bb", "G"]);
}
