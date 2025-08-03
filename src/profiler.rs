use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Performance profiler for tracking execution times and identifying bottlenecks
pub struct Profiler {
    timers: HashMap<String, Timer>,
    counters: HashMap<String, u64>,
    enabled: bool,
}

struct Timer {
    start_time: Instant,
    total_duration: Duration,
    call_count: u64,
    min_duration: Duration,
    max_duration: Duration,
}

impl Profiler {
    pub fn new(enabled: bool) -> Self {
        Self {
            timers: HashMap::new(),
            counters: HashMap::new(),
            enabled,
        }
    }

    /// Start timing a named operation
    pub fn start_timer(&mut self, name: &str) {
        if !self.enabled {
            return;
        }

        let timer = Timer {
            start_time: Instant::now(),
            total_duration: Duration::ZERO,
            call_count: 0,
            min_duration: Duration::MAX,
            max_duration: Duration::ZERO,
        };
        self.timers.insert(name.to_string(), timer);
    }

    /// Stop timing a named operation
    pub fn stop_timer(&mut self, name: &str) {
        if !self.enabled {
            return;
        }

        if let Some(timer) = self.timers.get_mut(name) {
            let duration = timer.start_time.elapsed();
            timer.total_duration += duration;
            timer.call_count += 1;
            timer.min_duration = timer.min_duration.min(duration);
            timer.max_duration = timer.max_duration.max(duration);
        }
    }

    /// Increment a counter
    pub fn increment_counter(&mut self, name: &str, value: u64) {
        if !self.enabled {
            return;
        }
        *self.counters.entry(name.to_string()).or_insert(0) += value;
    }

    /// Get timing statistics for a named operation
    pub fn get_timer_stats(&self, name: &str) -> Option<TimerStats> {
        self.timers.get(name).map(|timer| TimerStats {
            total_duration: timer.total_duration,
            call_count: timer.call_count,
            avg_duration: if timer.call_count > 0 {
                timer.total_duration / timer.call_count as u32
            } else {
                Duration::ZERO
            },
            min_duration: timer.min_duration,
            max_duration: timer.max_duration,
        })
    }

    /// Print a summary of all timing data
    pub fn print_summary(&self) {
        if !self.enabled || self.timers.is_empty() {
            return;
        }

        info!("=== Performance Profile Summary ===");

        // Sort timers by total duration (descending)
        let mut sorted_timers: Vec<_> = self.timers.iter().collect();
        sorted_timers.sort_by(|a, b| b.1.total_duration.cmp(&a.1.total_duration));

        for (name, timer) in sorted_timers {
            let avg_duration = if timer.call_count > 0 {
                timer.total_duration / timer.call_count as u32
            } else {
                Duration::ZERO
            };

            info!(
                "{}: {} calls, {:.2?} total, {:.2?} avg, {:.2?} min, {:.2?} max",
                name,
                timer.call_count,
                timer.total_duration,
                avg_duration,
                timer.min_duration,
                timer.max_duration
            );
        }

        if !self.counters.is_empty() {
            info!("=== Counters ===");
            for (name, count) in &self.counters {
                info!("{}: {}", name, count);
            }
        }
    }

    /// Reset all timers and counters
    pub fn reset(&mut self) {
        self.timers.clear();
        self.counters.clear();
    }

    /// Enable or disable profiling
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

#[derive(Debug, Clone)]
pub struct TimerStats {
    pub total_duration: Duration,
    pub call_count: u64,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
}

/// Macro for easy profiling of code blocks
#[macro_export]
macro_rules! profile_block {
    ($profiler:expr, $name:expr, $block:expr) => {{
        $profiler.start_timer($name);
        let result = $block;
        $profiler.stop_timer($name);
        result
    }};
}

/// Performance analyzer for identifying bottlenecks
pub struct PerformanceAnalyzer {
    profiler: Profiler,
    step_count: u32,
    last_report_step: u32,
    report_interval: u32,
}

impl PerformanceAnalyzer {
    pub fn new(enabled: bool, report_interval: u32) -> Self {
        Self {
            profiler: Profiler::new(enabled),
            step_count: 0,
            last_report_step: 0,
            report_interval,
        }
    }

    pub fn profiler(&mut self) -> &mut Profiler {
        &mut self.profiler
    }

    pub fn step(&mut self) {
        self.step_count += 1;

        if self.step_count - self.last_report_step >= self.report_interval {
            self.report_performance();
            self.last_report_step = self.step_count;
        }
    }

    fn report_performance(&self) {
        info!("=== Step {} Performance Report ===", self.step_count);
        self.profiler.print_summary();

        // Analyze potential bottlenecks
        self.analyze_bottlenecks();
    }

    fn analyze_bottlenecks(&self) {
        let mut slowest_operations = Vec::new();

        for (name, timer) in &self.profiler.timers {
            if timer.call_count > 0 {
                let avg_duration = timer.total_duration / timer.call_count as u32;
                slowest_operations.push((name.clone(), avg_duration, timer.call_count));
            }
        }

        slowest_operations.sort_by(|a, b| b.1.cmp(&a.1));

        if !slowest_operations.is_empty() {
            info!("=== Potential Bottlenecks ===");
            for (name, avg_duration, call_count) in slowest_operations.iter().take(5) {
                if *avg_duration > Duration::from_millis(1) {
                    warn!(
                        "Slow operation: {} (avg: {:.2?}, calls: {})",
                        name, avg_duration, call_count
                    );
                } else if *avg_duration > Duration::from_micros(100) {
                    info!(
                        "Moderate operation: {} (avg: {:.2?}, calls: {})",
                        name, avg_duration, call_count
                    );
                }
            }
        }
    }

    pub fn final_report(&self) {
        info!("=== Final Performance Report ===");
        self.profiler.print_summary();
        self.analyze_bottlenecks();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_profiler_timing() {
        let mut profiler = Profiler::new(true);

        profiler.start_timer("test_op");
        thread::sleep(Duration::from_millis(10));
        profiler.stop_timer("test_op");

        let stats = profiler.get_timer_stats("test_op").unwrap();
        assert_eq!(stats.call_count, 1);
        assert!(stats.total_duration >= Duration::from_millis(10));
    }

    #[test]
    fn test_profiler_counter() {
        let mut profiler = Profiler::new(true);

        profiler.increment_counter("test_counter", 5);
        profiler.increment_counter("test_counter", 3);

        assert_eq!(profiler.counters.get("test_counter"), Some(&8));
    }

    #[test]
    fn test_profile_block_macro() {
        let mut profiler = Profiler::new(true);

        let result = profile_block!(&mut profiler, "macro_test", {
            thread::sleep(Duration::from_millis(5));
            42
        });

        assert_eq!(result, 42);
        let stats = profiler.get_timer_stats("macro_test").unwrap();
        assert_eq!(stats.call_count, 1);
    }
}
