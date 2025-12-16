//! A signal is a collection of points that can be used to draw a line graph.

use num_traits::{cast::ToPrimitive, Bounded, Num};

pub(crate) struct Signal<T>
where
    T: Num,
{
    pub(crate) peak: T,
    pub(crate) max: T,
    pub(crate) points: std::collections::VecDeque<u64>,
}

impl<T: Num + Bounded> Signal<T> {
    pub(crate) fn with_capacity(capacity: usize, max: T) -> Self {
        Self {
            peak: T::zero(),
            max,
            points: std::collections::VecDeque::with_capacity(capacity),
        }
    }
}

impl<T: Num + ToPrimitive + PartialOrd + Copy> Signal<T> {
    pub(crate) fn push(&mut self, value: T) {
        self.peak = if self.peak > value { self.peak } else { value };

        if self.points.len() == self.points.capacity() {
            self.points.pop_front();
        }
        self.points.push_back(value.to_u64().unwrap_or(0));
        self.points.make_contiguous();
    }
}

impl<T: Num> Signal<T> {
    fn as_slice(&self) -> &[u64] {
        self.points.as_slices().0
    }

    /// Return the last n values as a u64 slice.
    pub(crate) fn as_slice_last_n(&self, n: usize) -> &[u64] {
        let len = self.points.len();
        if len < n {
            self.as_slice()
        } else {
            &self.as_slice()[len - n..]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_basics_u32() {
        let mut signal = Signal::<u32>::with_capacity(3, /* max */ 4);
        assert_eq!(signal.peak, 0);
        signal.push(1);
        signal.push(2);
        signal.push(3);

        assert_eq!(signal.as_slice(), &[1, 2, 3]);
        assert_eq!(signal.peak, 3);

        signal.push(4);
        assert_eq!(signal.as_slice(), &[2, 3, 4]);
        assert_eq!(signal.peak, 4);

        for _ in 0..10 {
            signal.push(1);
        }
        signal.push(0);
        assert_eq!(signal.as_slice(), &[1, 1, 0]);
        assert_eq!(signal.peak, 4);
    }

    #[test]
    fn test_signal_basics_f32() {
        let mut signal = Signal::<f32>::with_capacity(3, /* max */ 4.0);
        assert_eq!(signal.peak, 0.0);
        signal.push(1.0);
        signal.push(2.0);
        signal.push(3.0);

        assert_eq!(signal.as_slice(), &[1, 2, 3]);
        assert_eq!(signal.peak, 3.0);

        signal.push(4.0);
        assert_eq!(signal.as_slice(), &[2, 3, 4]);
        assert_eq!(signal.peak, 4.0);

        for _ in 0..10 {
            signal.push(1.0);
        }
        signal.push(0.0);
        assert_eq!(signal.as_slice(), &[1, 1, 0]);
        assert_eq!(signal.peak, 4.0);
    }
}
