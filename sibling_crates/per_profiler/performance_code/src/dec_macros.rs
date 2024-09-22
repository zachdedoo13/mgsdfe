
#[macro_export]
macro_rules! get_profiler {
    () => {
       performance_profiler::PROFILER.lock().unwrap()
    };
}