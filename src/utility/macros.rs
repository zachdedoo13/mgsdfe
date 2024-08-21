#[macro_export]
macro_rules! get {
    ($var: ident) => {
        *$var.lock().unwrap()
    };
}

#[macro_export]
macro_rules! init_static {
    ($name: ident: $ty:ty => $code:block) => {
        pub static $name: Lazy<Mutex<$ty>> = Lazy::new(|| Mutex::new($code));
    };
}