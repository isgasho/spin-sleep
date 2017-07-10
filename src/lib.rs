//! Accurate sleeping using `thread::sleep` & spinning
//!
//! The problem with `thread::sleep` is it isn't always very accurate, and this error can varies
//! on platform and state. Spinning is as accurate as we can get, but consumes the CPU
//! rather ungracefully.
//!
//! This library adds a middle ground, using a configurable native accuracy setting allowing
//! thread::sleep to wait the bulk of a sleep time, and spin the final section to guarantee
//! accuracy.
//!
//! # Examples
//! ```no_run
//! extern crate spin_sleep;
//! # use std::time::Duration;
//!
//! // Create a new sleeper that trusts native thread::sleep with 100μs accuracy
//! let spin_sleeper = spin_sleep::SpinSleeper::new(100_000);
//!
//! // Sleep for 1.01255 seconds, this will:
//! //  - thread:sleep for 1.01245 seconds, ie 100μs less than the requested duration
//! //  - spin until total 1.01255 seconds have elapsed
//! spin_sleeper.sleep(Duration::new(1, 12_550_000));
//! ```
//!
//! Sleep can also requested in f64 seconds (useful when used with `time` crate)
//!
//! ```no_run
//! # extern crate spin_sleep;
//! # use std::time::Duration;
//! # let spin_sleeper = spin_sleep::SpinSleeper::new(100_000);
//! spin_sleeper.sleep_s(1.01255);
//! ```

use std::thread;
use std::time::{Instant, Duration};

/// Accuracy container for spin sleeping
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpinSleeper {
    native_accuracy_ns: u32,
}

impl SpinSleeper {
    /// Constructs new SpinSleeper with the input native sleep accuracy
    /// The lower the `native_accuracy_ns` the more we effectively trust the accuracy of the
    /// `thread::sleep` function
    pub fn new(native_accuracy_ns: u32) -> SpinSleeper {
        SpinSleeper { native_accuracy_ns }
    }

    /// Returns configured native_accuracy_ns
    pub fn native_accuracy_ns(&self) -> u32 {
        self.native_accuracy_ns
    }

    /// Puts the current thread to sleep and then/or spins until the specified duration has elapsed.
    pub fn sleep(&self, duration: Duration) {
        let start = Instant::now();
        let accuracy = Duration::new(0, self.native_accuracy_ns);
        if duration > accuracy {
            thread::sleep(duration - accuracy)
        }
        // spin the rest of the duration
        while start.elapsed() < duration {}
    }

    /// Puts the current thread to sleep and then/or spins until the specified duration has elapsed.
    pub fn sleep_s(&self, seconds: f64) {
        self.sleep(Duration::new(
            seconds.floor() as u64,
            ((seconds % 1.0) * 1_000_000_000f64).round() as u32))
    }
}

#[cfg(test)]
mod spin_sleep_test {
    use super::*;

    // The worst case error is unbounded even when spinning, but this accuracy seems reasonable
    const ACCEPTABLE_DELTA_NS: u32 = 10_000;

    #[test]
    fn sleep_small() {
        let ns_duration = 12_345_678;

        let ps = SpinSleeper::new(20_000_000);
        ps.sleep(Duration::new(0, 1000)); // warm up

        let before = Instant::now();
        ps.sleep(Duration::new(0, ns_duration));
        let after = Instant::now();

        println!("Actual: {:?}", after.duration_since(before));
        assert!(after.duration_since(before) <= Duration::new(0, ns_duration + ACCEPTABLE_DELTA_NS));
        assert!(after.duration_since(before) >= Duration::new(0, ns_duration - ACCEPTABLE_DELTA_NS));
    }

    #[test]
    fn sleep_big() {
        let ns_duration = 212_345_678;

        let ps = SpinSleeper::new(20_000_000);
        ps.sleep(Duration::new(0, 1000)); // warm up

        let before = Instant::now();
        ps.sleep(Duration::new(1, ns_duration));
        let after = Instant::now();

        println!("Actual: {:?}", after.duration_since(before));
        assert!(after.duration_since(before) <= Duration::new(1, ns_duration + ACCEPTABLE_DELTA_NS));
        assert!(after.duration_since(before) >= Duration::new(1, ns_duration - ACCEPTABLE_DELTA_NS));
    }
}
