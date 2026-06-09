#![allow(dead_code)]
//! measurement_history.rs
//!
//! GUIでデータ履歴を表示する機能 を提供する

use crate::frame::Measurement;
use crate::ring_buffer::StaticRingBuffer;

// 公開API

pub struct MeasurementHistory {
    // とりあえず20件分で確保
    inner: StaticRingBuffer<Measurement, 20>,
}

impl MeasurementHistory {
    pub fn new() -> Self {
        Self {
            inner: StaticRingBuffer::new(),
        }
    }

    /// 追加
    pub fn add(&mut self, meas: Measurement) {
        self.inner.push(meas);
    }

    /// 履歴クリア
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// 空チェック
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// full ?
    pub fn is_full(&self) -> bool {
        self.inner.is_full()
    }

    /// 新しい順に返すiter
    pub fn iter_newest(&self) -> impl Iterator<Item = &Measurement> {
        self.inner.iter_newest()
    }
}

/// Defaultも実装
impl Default for MeasurementHistory {
    fn default() -> Self {
        Self::new()
    }
}
