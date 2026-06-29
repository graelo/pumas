//! A signal is a collection of points that can be used to draw a line graph.
//!
//! The implementation moved to [`crate::backend::history`] during the
//! iocraft migration; this module re-exports it so the (still-live) ratatui
//! frontend keeps compiling. Removed in Phase 3.

pub(crate) use crate::backend::history::Signal;

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
