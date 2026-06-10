//! `measurement_history.rs`
//!
//! # Measurement History Management Module
//!
//! This module provides domain-specific history retention services for decoded
//! `Measurement` packets, designed to feed historical data streams directly into the GUI view matrix.

use crate::frame::Measurement;
use crate::ring_buffer::StaticRingBuffer;

const HISTORY_SIZE: usize = 50;

pub struct MeasurementHistory {
    // 50件分で確保 (HISTORY_SIZEで指定)
    inner: StaticRingBuffer<Measurement, HISTORY_SIZE>,
}

/// Default
impl Default for MeasurementHistory {
    fn default() -> Self {
        Self::new()
    }
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

    #[allow(dead_code)]
    /// 履歴クリア
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// 空チェック
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[allow(dead_code)]
    /// full
    pub fn is_full(&self) -> bool {
        self.inner.is_full()
    }

    /// 新しい順に返すiter
    pub fn iter_newest(&self) -> impl Iterator<Item = &Measurement> {
        self.inner.iter_newest()
    }

    #[allow(dead_code)]
    /// 古い順に返すIter
    pub fn iter_oldest(&self) -> impl Iterator<Item = &Measurement> {
        self.inner.iter_oldest()
    }
}
