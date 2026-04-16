use std::collections::VecDeque;

use bevy::prelude::*;

use crate::prototype::rendering::scene::RecordableEntity;

pub const HISTORY_SECONDS: usize = 30;
pub const HISTORY_FPS: usize = 60;
pub const MAX_HISTORY_FRAMES: usize = HISTORY_SECONDS * HISTORY_FPS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackDirection {
    Rewind,
    Forward,
}

impl PlaybackDirection {
    pub fn frames_per_second(self) -> f32 {
        match self {
            Self::Rewind => 60.0,
            Self::Forward => 120.0,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct DreamLightSnapshot {
    pub anchor: Vec3,
    pub radius: f32,
    pub base_height: f32,
    pub speed: f32,
    pub intensity: f32,
    pub phase: f32,
}

#[derive(Clone, Debug, Default)]
pub struct PointLightSnapshot {
    pub intensity: f32,
    pub range: f32,
}

#[derive(Clone, Debug)]
pub struct EntitySnapshot {
    pub id: RecordableEntity,
    pub transform: Transform,
    pub point_light: Option<PointLightSnapshot>,
    pub dream_light: Option<DreamLightSnapshot>,
}

#[derive(Clone, Debug, Default)]
pub struct FrameSnapshot {
    pub entities: Vec<EntitySnapshot>,
}

impl FrameSnapshot {
    pub fn get(&self, id: RecordableEntity) -> Option<&EntitySnapshot> {
        self.entities.iter().find(|entity| entity.id == id)
    }
}

#[derive(Debug)]
pub struct FrameSample<'a> {
    pub from: &'a FrameSnapshot,
    pub to: &'a FrameSnapshot,
    pub blend: f32,
}

#[derive(Debug)]
pub struct HistoryBuffer {
    frames: VecDeque<FrameSnapshot>,
    max_frames: usize,
}

impl HistoryBuffer {
    pub fn new(max_frames: usize) -> Self {
        Self {
            frames: VecDeque::with_capacity(max_frames),
            max_frames,
        }
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn push(&mut self, frame: FrameSnapshot) {
        if self.frames.len() == self.max_frames {
            self.frames.pop_front();
        }
        self.frames.push_back(frame);
    }

    pub fn truncate_after(&mut self, index: usize) {
        while self.frames.len() > index.saturating_add(1) {
            self.frames.pop_back();
        }
    }

    pub fn sample(&self, cursor: f32) -> Option<FrameSample<'_>> {
        let last_index = self.frames.len().checked_sub(1)?;
        let clamped = cursor.clamp(0.0, last_index as f32);
        let start_index = clamped.floor() as usize;
        let end_index = (start_index + 1).min(last_index);
        let blend = if start_index == end_index {
            0.0
        } else {
            clamped - start_index as f32
        };

        Some(FrameSample {
            from: self.frames.get(start_index)?,
            to: self.frames.get(end_index)?,
            blend,
        })
    }
}

#[derive(Resource, Debug)]
pub struct TimeHistoryState {
    pub history: HistoryBuffer,
    pub cursor: f32,
    pub recording: bool,
    pub playback_direction: Option<PlaybackDirection>,
}

impl Default for TimeHistoryState {
    fn default() -> Self {
        Self {
            history: HistoryBuffer::new(MAX_HISTORY_FRAMES),
            cursor: 0.0,
            recording: true,
            playback_direction: None,
        }
    }
}

impl TimeHistoryState {
    pub fn newest_cursor(&self) -> f32 {
        self.history.len().saturating_sub(1) as f32
    }

    pub fn timeline_fraction(&self) -> f32 {
        let newest = self.newest_cursor();
        if newest <= 0.0 {
            1.0
        } else {
            (self.cursor / newest).clamp(0.0, 1.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(marker: f32) -> FrameSnapshot {
        FrameSnapshot {
            entities: vec![EntitySnapshot {
                id: RecordableEntity::Cube,
                transform: Transform::from_translation(Vec3::new(marker, 0.0, 0.0)),
                point_light: None,
                dream_light: None,
            }],
        }
    }

    #[test]
    fn push_evicts_oldest_frame_when_capacity_is_reached() {
        let mut history = HistoryBuffer::new(2);
        history.push(frame(1.0));
        history.push(frame(2.0));
        history.push(frame(3.0));

        assert_eq!(history.len(), 2);
        let sample = history.sample(0.0).unwrap();
        assert_eq!(
            sample
                .from
                .get(RecordableEntity::Cube)
                .unwrap()
                .transform
                .translation
                .x,
            2.0
        );
    }

    #[test]
    fn truncate_after_discards_future_frames() {
        let mut history = HistoryBuffer::new(4);
        history.push(frame(1.0));
        history.push(frame(2.0));
        history.push(frame(3.0));
        history.truncate_after(1);

        assert_eq!(history.len(), 2);
        let sample = history.sample(1.0).unwrap();
        assert_eq!(
            sample
                .from
                .get(RecordableEntity::Cube)
                .unwrap()
                .transform
                .translation
                .x,
            2.0
        );
    }

    #[test]
    fn sample_clamps_to_valid_range() {
        let mut history = HistoryBuffer::new(3);
        history.push(frame(4.0));
        history.push(frame(8.0));

        let sample = history.sample(99.0).unwrap();
        assert_eq!(sample.blend, 0.0);
        assert_eq!(
            sample
                .from
                .get(RecordableEntity::Cube)
                .unwrap()
                .transform
                .translation
                .x,
            8.0
        );
    }
}
