mod data;
mod engine;
mod rng;

use engine::Engine;
use std::cell::RefCell;

thread_local! {
    static ENGINE: RefCell<Option<Engine>> = const { RefCell::new(None) };
}

fn with_engine<R>(f: impl FnOnce(&mut Engine) -> R) -> R {
    ENGINE.with(|cell| {
        let mut borrow = cell.borrow_mut();
        let eng = borrow.as_mut().expect("engine not initialized");
        f(eng)
    })
}

#[no_mangle]
pub extern "C" fn init(seed: u32) {
    ENGINE.with(|cell| {
        *cell.borrow_mut() = Some(Engine::new(seed));
    });
}

#[no_mangle]
pub extern "C" fn start_stage(stage_idx: u32, timed_mode: u32, reset_run: u32) {
    with_engine(|e| e.start_stage(stage_idx as usize, timed_mode != 0, reset_run != 0, 1, false));
}

#[no_mangle]
pub extern "C" fn start_coop_stage(
    stage_idx: u32,
    timed_mode: u32,
    reset_run: u32,
    player_count: u32,
) {
    with_engine(|e| {
        e.start_stage(
            stage_idx as usize,
            timed_mode != 0,
            reset_run != 0,
            player_count as usize,
            true,
        )
    });
}

#[no_mangle]
pub extern "C" fn set_player_fish(player_idx: u32, fish_idx: u32) {
    with_engine(|e| e.set_player_fish(player_idx as usize, fish_idx as usize));
}

#[no_mangle]
pub extern "C" fn set_input(
    mouse_x: f32,
    mouse_y: f32,
    mouse_active: u32,
    key_mask: u32,
    dash_pressed: u32,
    special_pressed: u32,
) {
    with_engine(|e| {
        e.set_player_input(
            0,
            mouse_x,
            mouse_y,
            mouse_active != 0,
            key_mask,
            dash_pressed != 0,
            special_pressed != 0,
        );
    });
}

#[no_mangle]
pub extern "C" fn set_player_input(
    player_idx: u32,
    mouse_x: f32,
    mouse_y: f32,
    mouse_active: u32,
    key_mask: u32,
    dash_pressed: u32,
    special_pressed: u32,
) {
    with_engine(|e| {
        e.set_player_input(
            player_idx as usize,
            mouse_x,
            mouse_y,
            mouse_active != 0,
            key_mask,
            dash_pressed != 0,
            special_pressed != 0,
        );
    });
}

#[no_mangle]
pub extern "C" fn toggle_pause() {
    with_engine(|e| e.toggle_pause());
}

#[no_mangle]
pub extern "C" fn tick(dt: f32) {
    with_engine(|e| e.tick(dt));
}

#[no_mangle]
pub extern "C" fn get_init_ptr() -> *const u8 {
    with_engine(|e| e.init_json.as_ptr())
}

#[no_mangle]
pub extern "C" fn get_init_len() -> usize {
    with_engine(|e| e.init_json.len())
}

#[no_mangle]
pub extern "C" fn get_frame_ptr() -> *const f32 {
    with_engine(|e| e.frame_buf.as_ptr())
}

#[no_mangle]
pub extern "C" fn get_frame_len() -> usize {
    with_engine(|e| e.frame_buf.len())
}

#[no_mangle]
pub extern "C" fn get_hud_ptr() -> *const u8 {
    with_engine(|e| e.hud_json.as_ptr())
}

#[no_mangle]
pub extern "C" fn get_hud_len() -> usize {
    with_engine(|e| e.hud_json.len())
}
