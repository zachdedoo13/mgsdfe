#[macro_export]
macro_rules! get {
    ($var: ident) => {
        *$var.lock().unwrap()
    };
}

#[macro_export]
macro_rules! init_static {
    ($name: ident: $ty:ty => $code:block) => {
        pub static $name: once_cell::sync::Lazy<std::sync::Mutex<$ty>> = once_cell::sync::Lazy::new(|| std::sync::Mutex::new($code));
    };
}

#[macro_export]
macro_rules! timer {
    ($code: block) => {{
        let st = instant::Instant::now();
        let out = { $code };
        println!("Time elapsed -> {:?}", st.elapsed());
        out
    }};

    ($name: literal, $code: block) => {{
        let st = instant::Instant::now();
        let out = { $code };
        println!("{} -> {:?}", $name, st.elapsed());
        out
    }};
}