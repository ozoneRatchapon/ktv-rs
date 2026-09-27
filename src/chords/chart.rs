use super::types::{Chord, ChordChart, ChordMark, MERGE_SECS};

impl ChordChart {
    pub fn marks(&self) -> &[ChordMark] {
        &self.marks
    }

    pub fn is_empty(&self) -> bool {
        self.marks.is_empty()
    }

    /// Put `chord` at `at_secs` (clamped to 0, rounded to 0.01 s), keeping the chart sorted; a mark within
    /// [`MERGE_SECS`] is replaced.
    pub fn insert(&mut self, at_secs: f64, chord: Chord) {
        // Hundredths are finer than any tap; they keep the saved and copied JSON short
        let at_secs = (at_secs.max(0.0) * 100.0).round() / 100.0;
        let mark = ChordMark { at_secs, chord: chord.to_string() };
        match self.marks.iter().position(|m| (m.at_secs - at_secs).abs() < MERGE_SECS) {
            Some(i) => self.marks[i] = mark,
            None => {
                let i = self.marks.partition_point(|m| m.at_secs < at_secs);
                self.marks.insert(i, mark);
            }
        }
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.marks.len() {
            self.marks.remove(index);
        }
    }

    /// Indexes of the chord sounding at `secs` (none before the first mark) and of the next change.
    pub fn position(&self, secs: f64) -> (Option<usize>, Option<usize>) {
        let i = self.marks.partition_point(|m| m.at_secs <= secs);
        (i.checked_sub(1), (i < self.marks.len()).then_some(i))
    }

    /// Chords used so far, in order of first use: one-tap buttons for the next change.
    pub fn palette(&self) -> Vec<&str> {
        let mut seen = Vec::new();
        for mark in &self.marks {
            if !seen.contains(&mark.chord.as_str()) {
                seen.push(mark.chord.as_str());
            }
        }
        seen
    }

    /// Drop marks that are not a valid chord or time (hand-edited storage), re-sort, and merge near-duplicates.
    pub fn sanitized(self) -> Self {
        let mut clean = Self::default();
        for mark in self.marks {
            if let (true, Ok(chord)) = (mark.at_secs.is_finite(), mark.chord.parse::<Chord>()) {
                clean.insert(mark.at_secs, chord);
            }
        }
        clean
    }

    /// The chart as JSON for sharing or backup (`[{"at_secs":12.3,"chord":"Am"}, ...]`).
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}
