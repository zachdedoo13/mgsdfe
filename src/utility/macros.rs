#[macro_export]
macro_rules! render_pack_from_frame {
    ($name: ident, $frame: ident) => {
        let r_thing = $frame.wgpu_render_state().unwrap();
        let mut $name = RenderPack { device: &r_thing.device, queue: &r_thing.queue, renderer: &mut r_thing.renderer.write() };
    };
}



// egui mem macros
#[macro_export]
macro_rules! save_temp {
    ($ctx: ident, $name: literal, $data: expr) => {
        $ctx.memory_mut(|mem| mem.data.insert_temp($name.into(), $data))
    }
}

#[macro_export]
macro_rules! load_temp {
    ($ctx: ident, $name: literal, $or_else: expr) => {
        $ctx.memory(|mem| mem.data.get_temp::<_>($name.into()).unwrap_or_else(|| $or_else))
    }
}

#[macro_export]
macro_rules! save_persisted {
    ($ctx: ident, $name: literal, $data: expr) => {
        $ctx.memory_mut(|mem| mem.data.insert_persisted($name.into(), $data))
    }
}

#[macro_export]
macro_rules! load_persisted {
    ($ctx: ident, $name: literal, $or_else: expr) => {
        $ctx.memory_mut(|mem| mem.data.get_persisted::<_>($name.into()).unwrap_or_else(|| $or_else))
    }
}