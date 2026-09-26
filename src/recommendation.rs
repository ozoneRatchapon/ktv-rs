use crate::types::Song;
use std::collections::{HashMap, HashSet};

/// Telemetry recorded for each song played in the session.
#[derive(Clone, Debug, PartialEq)]
pub struct SongTelemetry {
    pub song_id: String,
    pub code: String,
    pub title: String,
    pub artist: String,
    pub category: String,
    pub duration_secs: u32,
    pub sang_seconds: f64,
    pub completed_natural: bool,
}

impl SongTelemetry {
    pub fn completion_ratio(&self) -> f32 {
        if self.duration_secs == 0 {
            return 1.0;
        }
        (self.sang_seconds as f32 / self.duration_secs as f32).clamp(0.0, 1.0)
    }

    /// True if the user spent significant time with the song (sang > 70% or finished naturally)
    pub fn is_high_affinity(&self) -> bool {
        self.completed_natural || self.completion_ratio() >= 0.70
    }

    /// True if the song was skipped very early (< 30 seconds or < 20% completion)
    pub fn is_early_skip(&self) -> bool {
        !self.completed_natural && (self.sang_seconds < 30.0 || self.completion_ratio() < 0.20)
    }
}

/// An anticipated recommendation slot, inspired by katgpt-sleep AnticipatedSlot.
#[derive(Clone, Debug, PartialEq)]
pub struct AnticipatedRecommendation {
    pub song: Song,
    pub predictability: f32, // Sigmoid-gated score [0.0, 1.0]
    pub reason: String,
}

/// Top Auto-DJ candidates, best first.
#[derive(Clone, Debug, PartialEq)]
pub struct AnticipatedRecommendationSet {
    pub candidates: Vec<AnticipatedRecommendation>,
}

/// Sigmoid function (katgpt-rs core invariant: sigmoid, never softmax)
#[inline]
pub fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

/// Sleep-Time Song Anticipator & Recommendation Engine.
/// Pre-computes next song candidates during playback ("sleep-time")
/// so wake-time selection ("when song ends") is instantaneous and zero-overhead.
#[derive(Clone, Debug)]
pub struct SleepTimeAnticipator {
    pub session_history: Vec<SongTelemetry>,
    pub genre_weights: HashMap<String, f32>,
    pub artist_weights: HashMap<String, f32>,
    pub played_song_ids: HashSet<String>,
}

impl Default for SleepTimeAnticipator {
    fn default() -> Self {
        Self::new()
    }
}

impl SleepTimeAnticipator {
    pub fn new() -> Self {
        Self {
            session_history: Vec::new(),
            genre_weights: HashMap::new(),
            artist_weights: HashMap::new(),
            played_song_ids: HashSet::new(),
        }
    }

    /// Record a finished or skipped song and update the latent session weights
    pub fn record_song_playback(&mut self, telemetry: SongTelemetry) {
        self.played_song_ids.insert(telemetry.song_id.clone());

        let ratio = telemetry.completion_ratio();
        if telemetry.is_early_skip() {
            // Tropical (max, +) Bottleneck Pruner:
            // Early skip triggers a hard bottleneck suppression (-infinity)
            self.genre_weights.insert(telemetry.category.clone(), f32::NEG_INFINITY);
            self.artist_weights.insert(telemetry.artist.clone(), f32::NEG_INFINITY);
        } else {
            let dwell_delta = if telemetry.is_high_affinity() {
                1.5 + ratio
            } else {
                0.5 * ratio
            };

            let g_entry = self.genre_weights.entry(telemetry.category.clone()).or_insert(0.0);
            if *g_entry != f32::NEG_INFINITY {
                *g_entry = (*g_entry + dwell_delta).clamp(-5.0, 10.0);
            }

            let a_entry = self.artist_weights.entry(telemetry.artist.clone()).or_insert(0.0);
            if *a_entry != f32::NEG_INFINITY {
                *a_entry = (*a_entry + dwell_delta * 1.2).clamp(-5.0, 10.0);
            }
        }

        self.session_history.push(telemetry);
    }

    /// Sleep-time compute: Pre-anticipates the top song recommendations from the catalog.
    pub fn sleep_compute(
        &self,
        catalog: &[Song],
        queued_song_ids: &HashSet<String>,
        current_song_id: Option<&str>,
        limit: usize,
    ) -> AnticipatedRecommendationSet {
        let mut scored_slots: Vec<AnticipatedRecommendation> = Vec::new();

        for song in catalog {
            // Constraint Pruning (from katgpt-rs ConstraintPruner trait):
            // 1. Prune currently playing song
            if let Some(curr_id) = current_song_id {
                if song.id == curr_id {
                    continue;
                }
            }
            // 2. Prune songs already waiting in the queue
            if queued_song_ids.contains(&song.id) {
                continue;
            }

            // Calculate base latent score
            let genre_weight = self.genre_weights.get(&song.category).copied().unwrap_or(0.0);
            let artist_weight = self.artist_weights.get(&song.artist).copied().unwrap_or(0.0);

            // Tropical Hard Bottleneck Prune:
            if genre_weight == f32::NEG_INFINITY || artist_weight == f32::NEG_INFINITY {
                continue;
            }

            // Recency penalty if played in current session
            let recency_penalty = if self.played_song_ids.contains(&song.id) {
                -3.0
            } else {
                0.0
            };

            // Dot-product composite score
            let raw_dot = (genre_weight * 0.45) + (artist_weight * 0.45) + recency_penalty;

            // Apply katgpt sigmoid gating (never softmax)
            let predictability = sigmoid(raw_dot);

            // Determine explanation reason
            let reason = if artist_weight > 1.0 {
                format!("Artist you love ({artist})", artist = song.artist)
            } else if genre_weight > 1.0 {
                format!("Your usual genre ({genre})", genre = song.category)
            } else if song.is_favorite {
                "Room favourite".to_string()
            } else {
                "Recommended for you".to_string()
            };

            scored_slots.push(AnticipatedRecommendation {
                song: song.clone(),
                predictability,
                reason,
            });
        }

        // Sort descending by predictability score
        scored_slots.sort_by(|a, b| {
            b.predictability
                .partial_cmp(&a.predictability)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        scored_slots.truncate(limit);
        AnticipatedRecommendationSet { candidates: scored_slots }
    }

    /// Wake-time lookup: Consumes the top anticipated candidate when the queue is empty.
    pub fn wake_consume(
        &self,
        anticipated_set: &AnticipatedRecommendationSet,
    ) -> Option<AnticipatedRecommendation> {
        anticipated_set.candidates.first().cloned()
    }
}
