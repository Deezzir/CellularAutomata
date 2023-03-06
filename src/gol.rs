use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget};

use crate::RGBA_HEX;

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
                    let r = rem_euclid((row + dr) as i32 - 1, self.board.len() as i32);
                    let c = rem_euclid((col + dc) as i32 - 1, self.board[0].len() as i32);
                    if self.board[r as usize][c as usize] == Cell::Alive {
                        n += 1;
                    }
                }
            }
        }

        n
    }
}

fn rem_euclid(a: i32, b: i32) -> i32 {
    let r = a % b;
    if (r < 0 && b > 0) || (r > 0 && b < 0) {
        r + b
    } else {
        r
    }
}
