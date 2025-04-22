use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// I'm building this system as a prototype for other progress based tools
/// Later on down the road. ETA is going to be a big part of
/// this system and have a good system that calculates estimated times
/// will be one of the foundations that makes this tool great.

// --- Progress ---
pub struct Progress {
    pub total: usize,
    pub progress: usize,
    pub starting_position: usize,
}

impl Progress {
    pub fn new(total: Option<usize>) -> Self {
        let total = total.unwrap_or(100);
        let starting_position = 0;
        let progress = starting_position;
        Progress {
            total,
            progress,
            starting_position,
        }
    }

    pub fn start(&mut self, at: Option<usize>) {
        let pos = at.unwrap_or(self.progress);
        self.starting_position = pos;
        self.progress = pos;
    }

    // Unused: warning: methods `finish` is never used
    // pub fn finish(&mut self) {
    //     self.progress = self.total;
    // }

    pub fn finished(&self) -> bool {
        self.progress == self.total
    }

    // Unused: warning: method `increment` is never used
    // pub fn increment(&mut self) {
    //     if self.progress == self.total {
    //         eprintln!(
    //             "WARNING: Your progress bar is currently at {} out of {}",
    //             self.progress, self.total
    //         );
    //     } else {
    //         self.progress += 1;
    //     }
    // }

    // Unused: warning: method `decrement` is never used
    // pub fn decrement(&mut self) {
    //     if self.progress == 0 {
    //         eprintln!(
    //             "WARNING: Your progress bar is currently at {} out of {}",
    //             self.progress, self.total
    //         );
    //     } else {
    //         self.progress -= 1;
    //     }
    // }

    // Unused: warning: method `reset` is never used
    // pub fn reset(&mut self) {
    //     self.start(Some(self.starting_position));
    // }

    pub fn set_progress(&mut self, new_progress: usize) {
        if new_progress > self.total {
            panic!("You can't set the item's current value to be greater than the total.");
        }
        self.progress = new_progress;
    }

    pub fn set_total(&mut self, new_total: usize) {
        if self.progress > new_total {
            println!("You can't set the item's total value to less than the current progress. Adjust progress to be eq to new total");
            self.set_progress(new_total);
        }
        self.total = new_total;
    }

    pub fn percentage_completed(&self) -> usize {
        if self.total == 0 {
            100
        } else {
            (self.progress * 100) / self.total
        }
    }

    // Unused: warning: method `percentage_completed_with_precision` is never used
    // pub fn percentage_completed_with_precision(&self) -> String {
    //     if self.total == 0 {
    //         "100.00".to_string()
    //     } else {
    //         let percent =
    //             (self.progress as f64 * 100.0 / self.total as f64 * 100.0).floor() / 100.0;
    //         format!("{:5.2}", percent)
    //     }
    // }

    /// Returns the “absolute” progress (progress minus starting position).
    // Unused: warning: method `absolute` is never used
    // pub fn absolute(&self) -> isize {
    //     self.progress as isize - self.starting_position as isize
    // }

    pub fn none(&self) -> bool {
        self.progress == 0
    }
}

// --- Timer ---
pub struct Timer {
    pub started_at: Option<SystemTime>,
    pub stopped_at: Option<SystemTime>,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            started_at: None,
            stopped_at: None,
        }
    }

    pub fn start(&mut self) {
        let now = SystemTime::now();
        if self.stopped() {
            // When resuming, adjust started_at to discount paused duration.
            if let (Some(started), Some(stopped)) = (self.started_at, self.stopped_at) {
                if let Ok(paused_duration) = stopped.duration_since(started) {
                    self.started_at = Some(now - paused_duration);
                } else {
                    self.started_at = Some(now);
                }
            } else {
                self.started_at = Some(now);
            }
        } else {
            self.started_at = Some(now);
        }
        self.stopped_at = None;
    }

    pub fn stop(&mut self) {
        if self.started() {
            self.stopped_at = Some(SystemTime::now());
        }
    }

    // Unused: warning: method `pause` is never used
    // pub fn pause(&mut self) {
    //     self.stop();
    // }

    // Unused: warning: method `resume` is never used
    // pub fn resume(&mut self) {
    //     self.start();
    // }

    pub fn started(&self) -> bool {
        self.started_at.is_some()
    }

    pub fn stopped(&self) -> bool {
        self.stopped_at.is_some()
    }

    // Unused: warning: method `reset` is never used
    // pub fn reset(&mut self) {
    //     self.started_at = None;
    //     self.stopped_at = None;
    // }

    pub fn is_reset(&self) -> bool {
        self.started_at.is_none()
    }

    // Unused: warning: method `restart` is never used
    // pub fn restart(&mut self) {
    //     self.reset();
    //     self.start();
    // }

    pub fn elapsed_seconds(&self) -> f64 {
        if let Some(started) = self.started_at {
            let end = self.stopped_at.unwrap_or_else(SystemTime::now);
            if let Ok(duration) = end.duration_since(started) {
                duration.as_secs_f64()
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    // Unused: warning: method `elapsed_whole_seconds` is never used
    // pub fn elapsed_whole_seconds(&self) -> u64 {
    //     self.elapsed_seconds().floor() as u64
    // }

    pub fn divide_seconds(seconds: u64) -> (u64, u64, u64) {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;
        (hours, minutes, secs)
    }
}

// --- Projector Trait & SmoothedAverage ---
// The trait now requires implementors to be Send + Sync.
pub trait Projector: Send + Sync {
    fn start(&mut self, at: Option<f64>);
    // Unused: warning: methods `decrement` is never used
    // fn decrement(&mut self);
    // Unused: warning: method `increment` is never used
    // fn increment(&mut self);
    fn set_progress(&mut self, new_progress: f64);
    // Unused: warning: method `reset` is never used
    // fn reset(&mut self);
    fn get_progress(&self) -> f64;
    fn none(&self) -> bool;
}

pub mod projectors {
    use super::Projector;

    pub struct SmoothedAverage {
        samples: [f64; 2],
        projection: f64,
        strength: f64,
    }

    // --- SmoothedAverage ---
    impl SmoothedAverage {
        pub const DEFAULT_STRENGTH: f64 = 0.1;
        // Unused: warning: associated constant `DEFAULT_BEGINNING_POSITION` is never used
        // pub const DEFAULT_BEGINNING_POSITION: f64 = 0.0;
        // Adjust the strength to make the system update the weighted average
        // more often or less often. Larger numbers will keep the current
        // value closer to the current projection, while lower numbers will
        // make the projection adjust more quickly based on changes.
        pub fn new(strength: Option<f64>, at: Option<f64>) -> Self {
            let strength = strength.unwrap_or(Self::DEFAULT_STRENGTH);
            let mut projector = SmoothedAverage {
                samples: [0.0, 0.0],
                projection: 0.0,
                strength,
            };
            projector.start(at);
            projector
        }

        fn absolute(&self) -> f64 {
            self.samples[1] - self.samples[0]
        }
        // Calculates a smoothed projection by blending the new value with the current projection.
        fn calculate(current_projection: f64, new_value: f64, rate: f64) -> f64 {
            new_value * (1.0 - rate) + current_projection * rate
        }
    }

    impl Projector for SmoothedAverage {
        fn start(&mut self, at: Option<f64>) {
            self.projection = 0.0;
            let initial = at.unwrap_or(self.get_progress());
            self.samples[0] = initial;
            self.samples[1] = initial;
        }

        // Unused: warning: method `decrement` is never used
        // fn decrement(&mut self) {
        //     let new_value = self.get_progress() - 1.0;
        //     self.set_progress(new_value);
        // }

        // Unused: warning: method `increment` is never used
        // fn increment(&mut self) {
        //     let new_value = self.get_progress() + 1.0;
        //     self.set_progress(new_value);
        // }

        fn set_progress(&mut self, new_progress: f64) {
            self.samples[1] = new_progress;
            self.projection = Self::calculate(self.projection, self.absolute(), self.strength);
        }

        // Unused: warning: method `reset` is never used
        // fn reset(&mut self) {
        //     self.start(Some(self.samples[0]));
        // }

        fn get_progress(&self) -> f64 {
            self.samples[1]
        }

        fn none(&self) -> bool {
            self.projection == 0.0
        }
    }

    pub fn from_type(
        name: Option<&str>,
        strength: Option<f64>,
        at: Option<f64>,
    ) -> Box<dyn Projector> {
        match name {
            Some("smoothed") => Box::new(SmoothedAverage::new(strength, at)),
            _ => Box::new(SmoothedAverage::new(strength, at)),
        }
    }
}

// --- Components ---
pub mod components {
    use super::{Progress, Projector, Timer};
    use std::sync::{Arc, Mutex};

    pub struct Percentage {
        pub progress: Arc<Mutex<Progress>>,
    }

    impl Percentage {
        pub fn new(progress: Arc<Mutex<Progress>>) -> Self {
            Percentage { progress }
        }

        pub fn percentage(&self) -> String {
            self.progress
                .lock()
                .unwrap()
                .percentage_completed()
                .to_string()
        }

        // Unused: warning: method `justified_percentage` is never used
        // pub fn justified_percentage(&self) -> String {
        //     format!(
        //         "{:>3}",
        //         self.progress.lock().unwrap().percentage_completed()
        //     )
        // }

        // Unused: warning: method `percentage_with_precision` is never used
        // pub fn percentage_with_precision(&self) -> String {
        //     self.progress
        //         .lock()
        //         .unwrap()
        //         .percentage_completed_with_precision()
        // }

        // Unused: warning: method `justified_percentage_with_precision` is never used
        // pub fn justified_percentage_with_precision(&self) -> String {
        //     format!(
        //         "{:>6}",
        //         self.progress
        //             .lock()
        //             .unwrap()
        //             .percentage_completed_with_precision()
        //     )
        // }
    }

    #[allow(dead_code)]
    pub struct Rate {
        pub rate_scale: Box<dyn Fn(f64) -> f64 + Send + Sync>,
        pub timer: Arc<Mutex<Timer>>,
        pub progress: Arc<Mutex<Progress>>,
    }

    impl Rate {
        pub fn new(timer: Arc<Mutex<Timer>>, progress: Arc<Mutex<Progress>>) -> Self {
            Rate {
                rate_scale: Box::new(|x| x),
                timer,
                progress,
            }
        }

        // Unused: warning: method `rate_of_change` is never used
        // pub fn rate_of_change(&self, _format_string: Option<&str>) -> String {
        //     let elapsed = self.timer.lock().unwrap().elapsed_seconds();
        //     if elapsed <= 0.0 {
        //         return "0".to_string();
        //     }
        //     let base_rate = self.progress.lock().unwrap().absolute() as f64 / elapsed;
        //     let scaled_rate = (self.rate_scale)(base_rate);
        //     format!("{}", scaled_rate)
        // }

        // Unused: warning: method `rate_of_change_with_precision` is never used
        // pub fn rate_of_change_with_precision(&self) -> String {
        //     self.rate_of_change(Some("%.2f"))
        // }
    }

    #[derive(Clone)]
    #[allow(dead_code)]
    pub enum OOBTimeFormat {
        Unknown,
        Friendly,
    }

    pub struct TimeComponent {
        pub timer: Arc<Mutex<Timer>>,
        pub progress: Arc<Mutex<Progress>>,
        pub projector: Arc<Mutex<Box<dyn Projector>>>,
    }

    impl TimeComponent {
        // const OOB_LIMIT_IN_HOURS: u64 = 99;
        // const OOB_UNKNOWN_TIME_TEXT: &'static str = "??:??:??";
        // const OOB_FRIENDLY_TIME_TEXT: &'static str = "> 4 Days";
        const NO_TIME_ELAPSED_TEXT: &'static str = "--:--:--";
        // const ESTIMATED_LABEL: &'static str = " ETA";
        // const ELAPSED_LABEL: &'static str = "Time";
        // const WALL_CLOCK_FORMAT: &'static str = "%H:%M:%S";

        pub fn new(
            timer: Arc<Mutex<Timer>>,
            progress: Arc<Mutex<Progress>>,
            projector: Arc<Mutex<Box<dyn Projector>>>,
        ) -> Self {
            TimeComponent {
                timer,
                progress,
                projector,
            }
        }

        pub fn estimated(&self, oob_format: Option<OOBTimeFormat>) -> String {
            if let Some(estimated_secs) = self.estimated_seconds_remaining() {
                let (hours, minutes, seconds) = Timer::divide_seconds(estimated_secs);
                if hours > 99 {
                    if let Some(oob) = oob_format {
                        return match oob {
                            OOBTimeFormat::Unknown => Self::NO_TIME_ELAPSED_TEXT.to_string(),
                            OOBTimeFormat::Friendly => Self::NO_TIME_ELAPSED_TEXT.to_string(),
                        };
                    }
                }
                self.format_time(hours, minutes, seconds)
            } else {
                Self::NO_TIME_ELAPSED_TEXT.to_string()
            }
        }

        fn format_time(&self, hours: u64, minutes: u64, seconds: u64) -> String {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        }

        // Unused: warning: method `estimated_with_label` is never used
        // pub fn estimated_with_label(&self, oob_format: Option<OOBTimeFormat>) -> String {
        //     format!("{}: {}", Self::ESTIMATED_LABEL, self.estimated(oob_format))
        // }

        // Unused: warning: method `elapsed` is never used
        // pub fn elapsed(&self) -> String {
        //     if !self.timer.lock().unwrap().started() {
        //         return Self::NO_TIME_ELAPSED_TEXT.to_string();
        //     }
        //     let elapsed = self.timer.lock().unwrap().elapsed_whole_seconds();
        //     let (hours, minutes, seconds) = Timer::divide_seconds(elapsed);
        //     self.format_time(hours, minutes, seconds)
        // }

        // Unused: warning: method `elapsed_with_label` is never used
        // pub fn elapsed_with_label(&self) -> String {
        //     format!("{}: {}", Self::ELAPSED_LABEL, self.elapsed())
        // }

        // Unused: warning: method `estimated_with_no_oob` is never used
        // pub fn estimated_with_no_oob(&self) -> String {
        //     self.estimated_with_elapsed_fallback(None)
        // }

        // Unused: warning: method `estimated_with_unknown_oob` is never used
        // pub fn estimated_with_unknown_oob(&self) -> String {
        //     self.estimated_with_elapsed_fallback(Some(OOBTimeFormat::Unknown))
        // }

        // Unused: warning: method `estimated_with_friendly_oob` is never used
        // pub fn estimated_with_friendly_oob(&self) -> String {
        //     self.estimated_with_elapsed_fallback(Some(OOBTimeFormat::Friendly))
        // }

        // Unused: warning: method `estimated_with_elapsed_fallback` is never used
        // fn estimated_with_elapsed_fallback(&self, oob_format: Option<OOBTimeFormat>) -> String {
        //     if self.progress.lock().unwrap().finished() {
        //         self.elapsed_with_label()
        //     } else {
        //         self.estimated_with_label(oob_format)
        //     }
        // }

        // Unused: warning: method `estimated_wall_clock` is never used
        // pub fn estimated_wall_clock(&self) -> String {
        //     if self.progress.lock().unwrap().finished() {
        //         if let Some(stopped) = self.timer.lock().unwrap().stopped_at {
        //             let datetime: DateTime<Local> = stopped.into();
        //             return datetime.format(Self::WALL_CLOCK_FORMAT).to_string();
        //         }
        //     }
        //     if !self.timer.lock().unwrap().started() {
        //         return Self::NO_TIME_ELAPSED_TEXT.to_string();
        //     }
        //     if let Some(estimated_secs) = self.estimated_seconds_remaining() {
        //         let estimated_time =
        //             SystemTime::now() + std::time::Duration::from_secs(estimated_secs);
        //         let datetime: DateTime<Local> = estimated_time.into();
        //         return datetime.format(Self::WALL_CLOCK_FORMAT).to_string();
        //     }
        //     Self::NO_TIME_ELAPSED_TEXT.to_string()
        // }

        fn estimated_seconds_remaining(&self) -> Option<u64> {
            let progress = self.progress.lock().unwrap();
            let projector_progress = self.projector.lock().unwrap().get_progress();
            if self.projector.lock().unwrap().none()
                || progress.none()
                || self.timer.lock().unwrap().stopped()
                || self.timer.lock().unwrap().is_reset()
            {
                return None;
            }
            let elapsed = self.timer.lock().unwrap().elapsed_seconds();
            if elapsed <= 0.0 || projector_progress == 0.0 {
                return None;
            }
            let total = progress.total as f64;
            let remaining = elapsed * ((total / projector_progress) - 1.0);
            Some(remaining.round() as u64)
        }
    }
}

// --- Base ---
/// The main ProgressTracker "Base" type.
#[allow(dead_code)]
pub struct Base {
    pub autostart: bool,
    pub autofinish: bool,
    pub finished: bool,
    pub timer: Arc<Mutex<Timer>>,
    pub projector: Arc<Mutex<Box<dyn Projector>>>,
    pub progress: Arc<Mutex<Progress>>,
    pub percentage_component: components::Percentage,
    pub rate_component: components::Rate,
    pub time_component: components::TimeComponent,
}

impl Base {
    pub fn new(options: Option<ProgressOptions>) -> Self {
        let opts = options.unwrap_or_default();
        let autostart = opts.autostart;
        let autofinish = opts.autofinish;
        let finished = false;

        let timer = Arc::new(Mutex::new(Timer::new()));
        let progress = Arc::new(Mutex::new(Progress::new(opts.total)));
        // Create the projector via the factory (using type, strength, and starting value).
        let proj_type = opts.projector_type.as_deref();
        let projector_obj =
            projectors::from_type(proj_type, opts.projector_strength, opts.projector_at);
        let projector = Arc::new(Mutex::new(projector_obj));

        // Create components (they share the same progress, timer, and projector).
        let percentage_component = components::Percentage::new(Arc::clone(&progress));
        let rate_component = components::Rate::new(Arc::clone(&timer), Arc::clone(&progress));
        let time_component = components::TimeComponent::new(
            Arc::clone(&timer),
            Arc::clone(&progress),
            Arc::clone(&projector),
        );

        let base = Base {
            autostart,
            autofinish,
            finished,
            timer,
            projector,
            progress,
            percentage_component,
            rate_component,
            time_component,
        };

        if base.autostart {
            // Start with the given starting_at value.
            let start_at = opts.starting_at.unwrap_or(0);
            base.start(Some(start_at));
        }

        base
    }

    pub fn start(&self, at: Option<usize>) {
        self.timer.lock().unwrap().start();
        self.progress.lock().unwrap().start(at);
        let val = self.progress.lock().unwrap().progress as f64;
        self.projector.lock().unwrap().start(Some(val));
    }

    // Unused: warning: method `finish` is never used
    // pub fn finish(&mut self) {
    //     if self.finished() {
    //         return;
    //     }
    //     self.finished = true;
    //     self.progress.lock().unwrap().finish();
    //     self.timer.lock().unwrap().stop();
    // }

    // Unused: warning: method `pause` is never used
    // pub fn pause(&self) {
    //     if !self.paused() {
    //         self.timer.lock().unwrap().pause();
    //     }
    // }

    // Unused: warning: method `stop` is never used
    // pub fn stop(&self) {
    //     if !self.stopped() {
    //         self.timer.lock().unwrap().stop();
    //     }
    // }

    // Unused: warning: method `resume` is never used
    // pub fn resume(&self) {
    //     if self.stopped() {
    //         self.timer.lock().unwrap().resume();
    //     }
    // }

    // Unused: warning: method `reset` is never used
    // pub fn reset(&mut self) {
    //     self.finished = false;
    //     self.progress.lock().unwrap().reset();
    //     self.projector.lock().unwrap().reset();
    //     self.timer.lock().unwrap().reset();
    // }

    // Unused: warning: method `stopped` is never used
    // pub fn stopped(&self) -> bool {
    //     self.timer.lock().unwrap().stopped() || self.finished()
    // }

    // Unused: warning: method `paused` is never used
    // pub fn paused(&self) -> bool {
    //     self.stopped()
    // }

    pub fn finished(&self) -> bool {
        self.finished || (self.autofinish && self.progress.lock().unwrap().finished())
    }

    // Unused: warning: method `started` is never used
    // pub fn started(&self) -> bool {
    //     self.timer.lock().unwrap().started()
    // }

    // Unused: warning: method `decrement` is never used
    // pub fn decrement(&self) {
    //     self.progress.lock().unwrap().decrement();
    //     self.projector.lock().unwrap().decrement();
    //     if self.finished() {
    //         self.timer.lock().unwrap().stop();
    //     }
    // }

    // Unused: warning: method `increment` is never used
    // pub fn increment(&self) {
    //     self.progress.lock().unwrap().increment();
    //     self.projector.lock().unwrap().increment();
    //     if self.finished() {
    //         self.timer.lock().unwrap().stop();
    //     }
    // }

    pub fn set_progress(&self, new_progress: usize) {
        self.progress.lock().unwrap().set_progress(new_progress);
        self.projector
            .lock()
            .unwrap()
            .set_progress(new_progress as f64);
        if self.finished() {
            self.timer.lock().unwrap().stop();
        }
    }

    pub fn set_total(&self, new_total: usize) {
        self.progress.lock().unwrap().set_total(new_total);
        if self.finished() {
            self.timer.lock().unwrap().stop();
        }
    }
}

// Options for initializing a Base instance.
#[derive(Default)]
pub struct ProgressOptions {
    pub total: Option<usize>,
    pub autostart: bool,
    pub autofinish: bool,
    pub starting_at: Option<usize>,
    pub projector_type: Option<String>,
    pub projector_strength: Option<f64>,
    pub projector_at: Option<f64>,
}

// Example usage:
// use progress_tracker::{Base, ProgressOptions};
//
// fn main() {
//     let options = ProgressOptions {
//         total: Some(200),
//         autostart: true,
//         autofinish: true,
//         starting_at: Some(0),
//         projector_type: Some("smoothed".to_string()),
//         projector_strength: Some(0.1),
//         projector_at: Some(0.0),
//     };
//
//     let pb = Base::new(Some(options));
//     pb.increment();
//     pb.increment();
//
//     println!(
//         "Progress: {}/{}",
//         pb.progress.lock().unwrap().progress,
//         pb.progress.lock().unwrap().total
//     );
//     println!("Percentage: {}", pb.percentage_component.percentage());
//     println!("Elapsed: {}", pb.time_component.elapsed_with_label());
// }
