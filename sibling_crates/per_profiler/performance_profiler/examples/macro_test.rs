use per_macros::time_function;
use performance_code::PROFILER;

fn main() {
   PROFILER.lock().unwrap().active = true;

   test(3);
}

#[time_function("MAIN")]
fn test(tt: i32) {
   println!("balls {tt}");
}