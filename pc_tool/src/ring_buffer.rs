//! `ring_buffer.rs`
//!
//! # Static Fixed-Size Ring Buffer Module
//!
//! A high-performance, zero-allocation ring buffer implementation with a fixed capacity
//! determined at compile time via Rust const generics.
//! This module is an internal engine component and is not part of the public API surface.
//!
//! ## Architectural Intent
//! This is a core engine component implemented as a fixed-size ring buffer using const generics.
//! It is not intended for direct external use, as public functionality is provided through a
//! higher-level wrapper.
//!
//! The implementation is fully stack-based and avoids heap allocation entirely.
//! It is designed for deterministic, small-capacity use cases such as streaming,
//! logging, and queueing.
//!
//! ## Target Performance Profile
//! Intended for thin stack environments, typically with capacities up to around 50 elements,
//! where predictable execution time and avoidance of heap allocation are critical.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StaticRingBuffer<T, const N: usize> {
    data: [Option<T>; N],
    write_index: usize,
    count: usize,
}

impl<T, const N: usize> Default for StaticRingBuffer<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> StaticRingBuffer<T, N> {
    // 0件は意味がない上，panicになるのでガードする
    // 上側チェックはなし。あまり巨大なのはスタックをひっ迫させるが
    // ～50件程度の使用を想定
    const _ASSERT_N: () = {
        assert!(N > 0, "StaticRingBuffer size N must be greater than 0");
    };
    pub(crate) fn new() -> Self {
        const { Self::_ASSERT_N };
        Self {
            data: [const { None }; N],
            write_index: 0,
            count: 0,
        }
    }

    /// 要素挿入 お尻に追加。リングなので満杯の場合は古いのから上書き
    pub(crate) fn push(&mut self, item: T) {
        self.data[self.write_index] = Some(item);
        self.write_index = (self.write_index + 1) % N;
        if self.count < N {
            self.count += 1;
        }
    }

    /// バッファクリア
    pub(crate) fn clear(&mut self) {
        for slot in self.data.iter_mut() {
            *slot = None;
        }
        self.data = [const { None }; N];
        self.write_index = 0;
        self.count = 0;
    }

    /// バッファの長さを返す
    pub(crate) fn len(&self) -> usize {
        self.count
    }

    /// バッファが空（要素数がゼロ）かどうかを返す
    pub(crate) fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// バッファが満杯（最大容量 N に達している）かどうかを返す
    pub(crate) fn is_full(&self) -> bool {
        self.count == N // ジェネリクスの最大サイズ N と比較
    }

    #[allow(dead_code)]
    /// iter 古い順をデフォルトに
    pub(crate) fn iter(&self) -> impl Iterator<Item = &T> {
        self.iter_oldest()
    }

    // 古い順に出力するイテレータ
    pub(crate) fn iter_oldest(&self) -> impl Iterator<Item = &T> {
        let start = if self.count == N { self.write_index } else { 0 };
        let count = self.count;

        (0..count).map(move |i| {
            let idx = (start + i) % N;
            // 内部状態破壊を検知するため expect()
            self.data[idx]
                .as_ref()
                .expect("RingBuffer: internal item missing")
        })
    }

    // 新しい順に出力するイテレータ
    pub(crate) fn iter_newest(&self) -> impl Iterator<Item = &T> {
        let start = if self.count == N { self.write_index } else { 0 };
        let count = self.count;

        (0..count).rev().map(move |i| {
            let idx = (start + i) % N;
            self.data[idx]
                .as_ref()
                .expect("RingBuffer: internal item missing")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overwrite_oldest() {
        let mut buf = StaticRingBuffer::<i32, 3>::new();

        buf.push(1);
        buf.push(2);
        buf.push(3);
        buf.push(4);

        let items: Vec<&i32> = buf.iter_newest().collect();

        assert_eq!(items, vec![&4, &3, &2]);
    }

    #[test]
    fn n_is_one() {
        let mut buf = StaticRingBuffer::<i32, 1>::new();

        buf.push(1);
        buf.push(2);
        buf.push(3);

        let items: Vec<&i32> = buf.iter_newest().collect();

        assert_eq!(items, vec![&3]);
    }

    #[test]
    fn buffer_partially_filled_and_empty() {
        let mut buf = StaticRingBuffer::<i32, 5>::new();

        // ケース1：完全に空（0件）のとき、イテレータは即座に終了するか？
        let items_0: Vec<&i32> = buf.iter_newest().collect();
        assert!(items_0.is_empty(), "0件のときは空のイテレータであるべき");

        // ケース2：1件だけ入っているとき（残り4マスは None の空インデックス）
        buf.push(100);
        let items_1: Vec<&i32> = buf.iter_newest().collect();
        assert_eq!(
            items_1,
            vec![&100],
            "1件のときは [100] が返るべき（None を踏まないこと）"
        );

        // ケース3：2件入っているとき（残り3マスは None の空インデックス）
        buf.push(200);
        let items_2: Vec<&i32> = buf.iter_newest().collect();
        // 最新順なので [200, 100]
        assert_eq!(
            items_2,
            vec![&200, &100],
            "2件のときは最新順で取得でき、None を踏まないこと"
        );

        // 古い順（時系列順）も一応チェック！
        let items_old: Vec<&i32> = buf.iter_oldest().collect();
        assert_eq!(
            items_old,
            vec![&100, &200],
            "古い順イテレータも正常に回ること"
        );
    }

    #[test]
    fn oldest_order_after_multiple_wraps() {
        let mut buf = StaticRingBuffer::<i32, 3>::new();

        for i in 1..=10 {
            buf.push(i);
        }

        let items: Vec<_> = buf.iter_oldest().copied().collect();
        assert_eq!(items, vec![8, 9, 10]);
    }

    #[test]
    fn iter_is_alias_of_oldest() {
        let mut buf = StaticRingBuffer::<i32, 3>::new();

        for i in 1..=10 {
            buf.push(i);
        }

        assert_eq!(
            buf.iter().copied().collect::<Vec<_>>(),
            buf.iter_oldest().copied().collect::<Vec<_>>()
        );
    }
}
