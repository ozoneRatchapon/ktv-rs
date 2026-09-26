use std::ops::RangeInclusive;

use super::types::{Booth, Placement, Requester};
use crate::types::{GuideTrack, QueueItem, Song};

/// Key shift limits in semitones.
pub const KEY_RANGE: RangeInclusive<i32> = -6..=6;

fn shift_key(key: i32, delta: i32) -> i32 {
    (key + delta).clamp(*KEY_RANGE.start(), *KEY_RANGE.end())
}

impl Booth {
    /// Add a song; returns `true` when it became the current song (a new take starts).
    pub fn add(&mut self, song: Song, requester: Requester, placement: Placement) -> bool {
        let item = QueueItem { queue_id: self.next_queue_id, song, key_shift: 0, requester: requester.label().to_string() };
        self.next_queue_id += 1;
        match (placement, self.current.is_some()) {
            (Placement::Next, true) => self.queue.insert(0, item),
            (Placement::Back, true) => self.queue.push(item),
            (Placement::Now, _) | (_, false) => {
                self.current = Some(item);
                return true;
            }
        }
        false
    }

    /// Move the head of the queue on stage; `false` (and nothing playing) when the queue is empty.
    pub fn advance(&mut self) -> bool {
        match self.queue.is_empty() {
            true => {
                self.current = None;
                false
            }
            false => {
                self.current = Some(self.queue.remove(0));
                true
            }
        }
    }

    pub fn shift_current_key(&mut self, delta: i32) {
        if let Some(curr) = &mut self.current {
            curr.key_shift = shift_key(curr.key_shift, delta);
        }
    }

    pub fn reset_current_key(&mut self) {
        if let Some(curr) = &mut self.current {
            curr.key_shift = 0;
        }
    }

    pub fn shift_item_key(&mut self, queue_id: u64, delta: i32) {
        if let Some(item) = self.queue.iter_mut().find(|it| it.queue_id == queue_id) {
            item.key_shift = shift_key(item.key_shift, delta);
        }
    }

    pub fn move_up(&mut self, index: usize) {
        if index > 0 && index < self.queue.len() {
            self.queue.swap(index, index - 1);
        }
    }

    pub fn move_down(&mut self, index: usize) {
        if index + 1 < self.queue.len() {
            self.queue.swap(index, index + 1);
        }
    }

    pub fn remove(&mut self, queue_id: u64) {
        self.queue.retain(|it| it.queue_id != queue_id);
    }

    pub fn clear_queue(&mut self) {
        self.queue.clear();
    }

    /// Put a guide on every copy of a song on stage or in the queue.
    pub fn set_guide(&mut self, song_id: &str, guide: Option<&GuideTrack>) {
        let items = self.current.iter_mut().chain(self.queue.iter_mut());
        for item in items.filter(|it| it.song.id == song_id) {
            item.song.guide = guide.cloned();
        }
    }
}
