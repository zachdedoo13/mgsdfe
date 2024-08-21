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

#[macro_export]
macro_rules! render_pack_from_frame {
    ($name: ident, $frame: ident) => {
        let r_thing = $frame.wgpu_render_state().unwrap();
        let mut $name = RenderPack { device: &r_thing.device, queue: &r_thing.queue, renderer: &mut r_thing.renderer.write() };
    };
}