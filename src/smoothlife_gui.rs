use rand::Rng;
use raylib::consts::{TextureFilter, TextureWrap};
use raylib::core::texture::Image;
use raylib::prelude::*;

// RayLib constants
const SCREEN_WIDTH: i32 = 1600;
const SCREEN_HEIGHT: i32 = 900;
const FPS: u32 = 60;
const SHADER: &str = include_str!("static/smoothlife.fs");
const SCALAR: f32 = 0.8;

struct Board {
    image: Image,
}

#[allow(dead_code)]
impl Board {
    fn new(h: i32, w: i32) -> Board {
        Board {
            image: Image::gen_image_color(w, h, Color::BLACK),
        }
    }

    fn randomize(&mut self) {
        let mut rng = rand::thread_rng();
        for y in 0..self.image.height {
            for x in 0..self.image.width {
                let c: u8 = rng.gen();
                let color = Color::new(c, c, c, 255);
                self.image.draw_pixel(x, y, color);
            }
        }
    }

    fn randomize_perlin_noize(&mut self) {
        self.image = Image::gen_image_perlin_noise(self.image.width, self.image.height, 0, 0, 4.0)
    }

    fn get_image(&self) -> &Image {
        &self.image
    }
}

fn create_render_texture(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    width: u32,
    height: u32,
) -> RenderTexture2D {
    let t = rl.load_render_texture(thread, width, height).unwrap();
    t.set_texture_wrap(thread, TextureWrap::TEXTURE_WRAP_CLAMP);
    t.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);

    t
}

fn main() {
    let h = (SCREEN_HEIGHT as f32 * SCALAR) as i32;
    let w = (SCREEN_WIDTH as f32 * SCALAR) as i32;

    // Board Setup
    let mut board = Board::new(h, w);
    // board.randomize();
    board.randomize_perlin_noize();

    // RayLib setup
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("SmoothLife")
        .build();
    rl.set_target_fps(FPS);

    // Texture setup
    let texture = rl
        .load_texture_from_image(&thread, board.get_image())
        .unwrap();

    // RenderTexture setup
    let mut state0 = create_render_texture(&mut rl, &thread, w as u32, h as u32);
    let mut state1 = create_render_texture(&mut rl, &thread, w as u32, h as u32);

    {
        let mut d = rl.begin_drawing(&thread);
        let mut d = d.begin_texture_mode(&thread, &mut state0);
        d.clear_background(Color::BLACK);
        d.draw_texture(&texture, 0, 0, Color::WHITE);
    }

    // Shader setup
    let mut shader = rl.load_shader_from_memory(&thread, None, Some(SHADER));
    let loc = shader.get_shader_location("resolution");
    shader.set_shader_value(loc, [texture.width as f32, texture.height as f32]);

    // Main loop
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture_ex(&state0, Vector2::zero(), 0.0, 1.0 / SCALAR, Color::WHITE);

        {
            let mut d = d.begin_texture_mode(&thread, &mut state1);
            let mut d = d.begin_shader_mode(&shader);
            d.draw_texture(&state0, 0, 0, Color::WHITE);
        }

        // Swap states
        std::mem::swap(&mut state0, &mut state1);
    }
}
