

/// returns a mutable access to profiler, do not assign to a variable 
#[macro_export]
macro_rules! get_profiler {
    () => {
       performance_profiler::PROFILER.write().unwrap()
    };
}