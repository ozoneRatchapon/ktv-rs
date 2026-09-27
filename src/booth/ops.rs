use super::types::{Booth, Placement, Requester};
use crate::types::{GuideTrack, QueueItem, Song};

impl Booth {
    /// Add a song; returns `true` when it became the current song (a new take starts).
    pub fn add(&mut self, song: Song, requester: Requester, placement: Placement) -> bool {
        let item = QueueItem { queue_id: self.next_queue_id, song, requester: requester.label().to_string() };
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
