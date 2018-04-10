// use std::{sync::Mutex, time::{Duration, Instant}};

// pub struct Ticker {
//     ticks: Mutex<u32>,
//     start: Mutex<Instant>,
// }

// impl Ticker {
//     pub fn new() -> Self {
//         Ticker {
//             ticks: Mutex::new(0),
//             start: Mutex::new(Instant::now()),
//         }
//     }

//     pub fn tick(&self) {
//         let mut count = self.ticks.lock().unwrap();
//         *count += 1;
//     }

//     pub fn ticks_per_second(&self) -> f64 {
//         self.ticks() as f64 / duration_to_f64(self.elapsed())
//     }

//     fn ticks(&self) -> u32 {
//         *self.ticks.lock().unwrap()
//     }

//     pub fn elapsed(&self) -> Duration {
//         self.start.lock().unwrap().elapsed()
//     }

//     pub fn reset(&self) {
//         let mut ticks = self.ticks.lock().unwrap();
//         *ticks = 0;

//         let mut start = self.start.lock().unwrap();
//         *start = Instant::now();
//     }
// }

// // Copied from ggez source
// pub fn duration_to_f64(d: Duration) -> f64 {
//     let seconds = d.as_secs() as f64;
//     let nanos = f64::from(d.subsec_nanos());
//     seconds + (nanos * 1e-9)
// }

// test_type_impls!(ticker_is_sync, Ticker, Sync);
// test_type_impls!(ticker_is_send, Ticker, Send);
