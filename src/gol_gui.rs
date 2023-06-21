use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget};
use sdl2::video::Window;

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

const MAX_ROWS: u16 = 125;
const MAX_COLS: u16 = 125;
const MIN_ROWS: u16 = 10;
const MIN_COLS: u16 = 10;

type Rows = usize;
type Cols = usize;

#[derive(Clone, PartialEq)]
enum Cell {
    Dead,
    Alive,
}

impl Cell {
    fn as_color_hex(&self) -> u32 {
        match self {
            Cell::Alive => 0xFFFFFFFF,
            Cell::Dead => 0x000000FF,
        }
    }

    fn toggle(&mut self) {
        match self {
            Cell::Alive => *self = Cell::Dead,
            Cell::Dead => *self = Cell::Alive,
        }
    }
}

pub struct Board {
    board: Vec<Vec<Cell>>,
    cursor: (Cols, Rows),
}

impl Board {
    pub fn new(rows: Rows, cols: Cols) -> Self {
        let rows = rows.clamp(MIN_ROWS as usize, MAX_ROWS as usize);
        let cols = cols.clamp(MIN_COLS as usize, MAX_COLS as usize);

        Self {
            board: vec![vec![Cell::Dead; cols]; rows],
            cursor: (0, 0),
        }
    }

    pub fn next_gen(&mut self) {
        let mut new_board = self.board.clone();

        for (ir, row) in new_board.iter_mut().enumerate() {
            for (ic, item) in row.iter_mut().enumerate() {
                let n = self.count_n(ir, ic);
                match self.board[ir][ic] {
                    Cell::Dead => *item = if n == 3 { Cell::Alive } else { Cell::Dead },
                    Cell::Alive => {
                        *item = if (2..=3).contains(&n) {
                            Cell::Alive
                        } else {
                            Cell::Dead
                        }
                    }
                }
            }
        }

        self.board = new_board;
    }

    pub fn toggle_cur_cell(&mut self) {
        let (c, r) = self.cursor;
        self.board[r][c].toggle();
    }

    pub fn clear(&mut self) {
        self.board = Self::new(self.board.len(), self.board[0].len()).board;
    }

    pub fn draw<T: RenderTarget>(&self, c: &mut Canvas<T>, width: u32, height: u32) {
        let cell_h = height as i32 / self.board.len() as i32;
        let cell_w = width as i32 / self.board[0].len() as i32;

        for (ir, row) in self.board.iter().enumerate() {
            for (ic, item) in row.iter().enumerate() {
                let x = ic as i32 * cell_w;
                let y = ir as i32 * cell_h;

                let rect = Rect::new(x, y, cell_w as u32, cell_h as u32);
                c.set_draw_color(RGBA_HEX!(item.as_color_hex()));
                c.fill_rect(rect).unwrap();
            }
        }
    }

    pub fn randomize(&mut self) {
        for row in self.board.iter_mut() {
            for item in row.iter_mut() {
                *item = if rand::random() {
                    Cell::Dead
                } else {
                    Cell::Alive
                }
            }
        }
    }

    fn count_n(&self, row: Rows, col: Cols) -> usize {
        let mut n: usize = 0;

        for dr in 0..=2 {
            for dc in 0..=2 {
                if dr != 1 || dc != 1 {
                    let r = emod((row + dr) as i32 - 1, self.board.len() as i32);
                    let c = emod((col + dc) as i32 - 1, self.board[0].len() as i32);
                    if self.board[r as usize][c as usize] == Cell::Alive {
                        n += 1;
                    }
                }
            }
        }

        n
    }
}

fn emod(a: i32, b: i32) -> i32 {
    (a % b + b) % b
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

    let mut board = Board::new(ROWS, COLS);
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
            if r_timeout <= 0.0 {
                r_timeout = RENDER_TIMEOUT;
                board.next_gen();
            }
        }

        canvas.clear();
        board.draw(&mut canvas, width, height);
        canvas.present();
    }

    Ok(())
}
