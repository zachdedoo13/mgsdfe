mod profiler;
pub mod dec_macros;

use std::sync::RwLock;
use lazy_static::lazy_static;
pub use profiler::{PerformanceProfiler, FunctionProfile};


lazy_static! {
    /// only use with the macro ``get_profiler!()``
    pub static ref PROFILER: RwLock<PerformanceProfiler> = RwLock::new(PerformanceProfiler::default());
}



#[cfg(test)]
mod tests {
    use std::thread::sleep;
    use std::time::Duration;
    use crate::profiler::PerformanceProfiler;

    #[test]
    fn timing_test() {
        let mut per = PerformanceProfiler::default();



        for _ in 0..1000 {
            per.resolve_profiler();
            per.start_time_function("MAIN");

            sleep(Duration::from_millis(2));

            per.end_time_function("MAIN");
        }

        println!("{:?}", per.profiles.get("MAIN"));
    }
}