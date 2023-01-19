use std::io::Write;

use termion::style;
use termion::{clear, color, cursor};

use crate::MAX_COLS;
use crate::MAX_ROWS;
use crate::MIN_COLS;
use crate::MIN_ROWS;

const HIGHLIGHT_PAIR: (&dyn color::Color, &dyn color::Color) = (&color::Black, &color::White);

type Rows = usize;
type Cols = usize;

#[derive(Clone, Copy, PartialEq)]
enum RenderMode {
    Ascii,
    Unicode,
}

#[derive(Clone, PartialEq)]
enum Cell {
    Dead,
    Alive,
}

impl Cell {
    fn as_str(&self, mode: RenderMode) -> &str {
        match mode {
            RenderMode::Ascii => match self {
                Cell::Alive => "@",
                Cell::Dead => "-",
            },
            RenderMode::Unicode => match self {
                Cell::Alive => "▢",
                Cell::Dead => "■",
            },
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
    render_mode: RenderMode,
    cursor: (Cols, Rows),
}

impl Board {
    pub fn new(rows: Rows, cols: Cols) -> Self {
        let rows = rows.clamp(MIN_ROWS as usize, MAX_ROWS as usize);
        let cols = cols.clamp(MIN_COLS as usize, MAX_COLS as usize);

        Self {
            board: vec![vec![Cell::Dead; cols]; rows],
            render_mode: RenderMode::Ascii,
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

    pub fn to_ascii_mode(&mut self) {
        self.render_mode = RenderMode::Ascii;
    }

    pub fn to_unicode_mode(&mut self) {
        self.render_mode = RenderMode::Unicode;
    }

    pub fn move_cursor_left(&mut self) {
        self.cursor.0 = self.cursor.0.saturating_sub(1);
    }

    pub fn move_cursor_right(&mut self) {
        self.cursor.0 = (self.cursor.0 + 1).clamp(0, self.board[0].len() - 1);
    }

    pub fn move_cursor_up(&mut self) {
        self.cursor.1 = self.cursor.1.saturating_sub(1);
    }

    pub fn move_cursor_down(&mut self) {
        self.cursor.1 = (self.cursor.1 + 1).clamp(0, self.board.len() - 1);
    }

    pub fn toggle_cur_cell(&mut self) {
        let (c, r) = self.cursor;
        self.board[r][c].toggle();
    }

    pub fn render<W: Write>(&self, s: &mut W) {
        write!(s, "{}{}", cursor::Goto(1, 1), clear::AfterCursor).unwrap();

        for (ir, row) in self.board.iter().enumerate() {
            write!(s, "{}", cursor::Goto(1, (ir + 1) as u16)).unwrap();

            for (ic, item) in row.iter().enumerate() {
                write!(s, "{}", if ic == 0 { " " } else { "" }).unwrap();
                write!(s, "{}", item.as_str(self.render_mode)).unwrap();
                write!(s, "{}", if ic < row.len() - 1 { " " } else { "" }).unwrap();
            }
            writeln!(s, "").unwrap();
        }

        self.highlight_cursor(s);
    }

    fn highlight_cursor<W: Write>(&self, s: &mut W) {
        if self.render_mode == RenderMode::Ascii {
            let (c, r) = self.cursor;
            let state = self.board[r][c].as_str(self.render_mode);

            write!(
                s,
                "{}{}",
                color::Fg(HIGHLIGHT_PAIR.0),
                color::Bg(HIGHLIGHT_PAIR.1)
            )
            .unwrap();
            write!(s, "{}[", cursor::Goto((c * 2 + 1) as u16, (r + 1) as u16)).unwrap();
            write!(
                s,
                "{}{}",
                cursor::Goto((c * 2 + 2) as u16, (r + 1) as u16),
                state
            )
            .unwrap();
            write!(s, "{}]", cursor::Goto((c * 2 + 3) as u16, (r + 1) as u16)).unwrap();
            write!(s, "{}", style::Reset).unwrap();
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
