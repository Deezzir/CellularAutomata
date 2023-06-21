mod cautomata;

use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use cautomata::board::*;
use cautomata::gol::*;

const WINDOW_HEIGHT: u32 = 800;
const WINDOW_WIDHT: u32 = 1000;
const SCREEN_FPS: u32 = 60;
const DELTA_TIME: f32 = 1.0 / SCREEN_FPS as f32;
const RENDER_TIMEOUT: f32 = 0.2f32;

const ROWS: usize = 100;
const COLS: usize = 100;

#[macro_export]
macro_rules! RGBA_HEX {
    ($hex:expr) => {{
        let r = (($hex >> 8 * 3) & 0xFF) as u8;
        let g = (($hex >> 8 * 2) & 0xFF) as u8;
        let b = (($hex >> 8 * 1) & 0xFF) as u8;
        let a = (($hex >> 8 * 0) & 0xFF) as u8;
        Color::RGBA(r, g, b, a)
    }};
}

fn sdl_error(err: String) -> String {
    format!("[SDL ERROR]: {err}.")
}

fn sdl_create_window(sdl_ctx: &sdl2::Sdl) -> Result<Window, String> {
    let video_subsys = sdl_ctx.video().map_err(|err| sdl_error(err))?;

    video_subsys
        .window("GoLrs", WINDOW_WIDHT, WINDOW_HEIGHT)
        .position_centered()
        .resizable()
        .allow_highdpi()
        .build()
        .map_err(|err| sdl_error(err.to_string()))
}

fn sdl_create_canvas(window: Window) -> Result<Canvas<Window>, String> {
    let mut canvas = window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .map_err(|err| sdl_error(err.to_string()))?;

    canvas
        .set_logical_size(WINDOW_WIDHT, WINDOW_HEIGHT)
        .map_err(|err| sdl_error(err.to_string()))?;

    Ok(canvas)
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init().map_err(|err| sdl_error(err))?;
    let window = sdl_create_window(&sdl_context)?;
    let mut canvas = sdl_create_canvas(window)?;

    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    canvas.present();

    let mut board = GoL::new(ROWS, COLS);
    let mut pause = false;
    let (mut width, mut height) = canvas.window().size();
    let mut r_timeout = RENDER_TIMEOUT;

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::Window {
                    win_event: WindowEvent::SizeChanged(w, h),
                    ..
                } => {
                    width = w as u32;
                    height = h as u32;
                    canvas.set_viewport(Rect::new(0, 0, width, height));
                }
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => board.randomize(),
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => board.clear(),
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => pause = !pause,
                _ => {}
            }
        }

        if !pause {
            r_timeout -= DELTA_TIME;
        }

        if r_timeout <= 0.0 {
            r_timeout = RENDER_TIMEOUT;
            board.next_gen();
        }

        canvas.clear();
        board.draw(&mut canvas, width, height);
        canvas.present();
    }

    Ok(())
}
