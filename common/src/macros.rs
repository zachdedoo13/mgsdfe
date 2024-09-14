/// returns an immutable singleton
/// can be used directly or with
///
/// ``` let foo = &Get!(x); ```
#[macro_export]
macro_rules! get {
    ($var: ident) => {
        *$var.lock().unwrap().as_ref().expect(" Not initulized")
    };
}

/// returns a singleton reference
/// as a &reference, cannot
/// be used inline
///
/// ``` get_ref!(SETTINGS, settings); let v = settings.time ```
#[macro_export]
macro_rules! get_ref {
    ($var: ident, $name: ident) => {
        let binding = $var.lock().unwrap();
        let $name = binding.as_ref().unwrap();
    };
}

/// returns a mutable singleton
/// can be used directly or with
///
/// ``` let foo = &mut Get!(x); ```
#[macro_export]
macro_rules! get_mut {
    ($var: ident) => {
        *$var.lock().unwrap().as_mut().expect("Not initulized")
    };
}

/// returns a mutable singleton
/// as a &mut reference, cannot
/// be used inline
///
/// ``` get_mut_ref!(SETTINGS, settings); settings.time = 0.0 ```
#[macro_export]
macro_rules! get_mut_ref {
    ($var: ident, $name: ident) => {
        let mut binding = $var.lock().unwrap();
        let $name = binding.as_mut().unwrap();
    };
}


/// inits a singleton
///
/// ``` init_static!(FOO: Foo => { Foo::new() }); ```
#[macro_export]
macro_rules! init_static {
    ($name: ident: $ty:ty => $code:block) => {
        pub static $name: once_cell::sync::Lazy<std::sync::Mutex<Option<$ty>>>
            = once_cell::sync::Lazy::new(|| std::sync::Mutex::new(Some($code)));
    };
}


/// inits a static as None, must be set before use
///
#[macro_export]
macro_rules! init_none_static {
    ($name: ident: $ty:ty) => {
        pub static $name: once_cell::sync::Lazy<std::sync::Mutex<Option<$ty>>> = once_cell::sync::Lazy::new(|| std::sync::Mutex::new(None));
    };
}

/// sets the value of a none static
///
/// ```  ```
#[macro_export]
macro_rules! set_none_static {
    ($var: ident => $code:block) => {
        *$var.lock().unwrap() = Some($code)
    };
}


/// is timer
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