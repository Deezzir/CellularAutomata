use std::cmp::max;
use std::io::{stdin, stdout, Write};
use std::sync::mpsc;
use std::time::Duration;
use std::{fmt, thread};

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, cursor};

const ROWS: usize = 20;
const COLS: usize = 20;

#[derive(Clone, PartialEq)]
enum Cell {
    Dead,
    Alive,
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Cell::Alive => write!(f, "▢"),
            Cell::Dead => write!(f, "■"),
        }
    }
}

struct Board {
    board: Vec<Vec<Cell>>,
    rows: usize,
    cols: usize,
}

impl Board {
    fn new(rows: usize, cols: usize) -> Self {
        let rows = max(rows, 10);
        let cols = max(cols, 10);

        let mut board = vec![vec![Cell::Dead; COLS]; ROWS];
        // male a glider
        board[0][1] = Cell::Alive;
        board[1][2] = Cell::Alive;
        board[2][0] = Cell::Alive;
        board[2][1] = Cell::Alive;
        board[2][2] = Cell::Alive;

        Self { board, rows, cols }
    }

    fn count_n(&mut self, row: usize, col: usize) -> usize {
        let mut n: usize = 0;

        for dr in 0..=2 {
            for dc in 0..=2 {
                if dr != 1 || dc != 1 {
                    let r = rem_euclid((row + dr) as i32 - 1, self.rows as i32);
                    let c = rem_euclid((col + dc) as i32 - 1, self.cols as i32);
                    if self.board[r as usize][c as usize] == Cell::Alive {
                        n += 1;
                    }
                }
            }
        }
        return n;
    }

    fn next(&mut self) {
        let mut new_board = self.board.clone();

        for row in 0..self.rows {
            for col in 0..self.cols {
                let n = self.count_n(row, col);
                match self.board[row][col] {
                    Cell::Dead => {
                        new_board[row][col] = if n == 3 { Cell::Alive } else { Cell::Dead }
                    }
                    Cell::Alive => {
                        new_board[row][col] = if (2..=3).contains(&n) {
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

    fn render_board<W: Write>(&mut self, s: &mut W) {
        write!(s, "{}{}", cursor::Goto(1, 1), clear::AfterCursor).unwrap();
        write!(s, " {} ", "▁".repeat(self.rows * 2).as_str()).unwrap();

        for row in 0..self.rows {
            write!(s, "{}▕", cursor::Goto(1, (row + 2) as u16)).unwrap();
            for col in 0..self.cols {
                write!(s, "{} ", self.board[row][col]).unwrap();
            }
            writeln!(s, "▎").unwrap();
        }

        writeln!(s, "{}", cursor::Goto(1, self.rows as u16 + 1)).unwrap();
        write!(s, " {} ", "▔".repeat(self.rows * 2).as_str()).unwrap();
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

fn main() {
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", cursor::Hide).unwrap();

    let timeout = Duration::from_millis(100);
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut keys = stdin().keys();
        while let Some(Ok(key)) = keys.next() {
            tx.send(key).unwrap();
        }
    });

    let mut quit = false;
    let mut board = Board::new(ROWS, COLS);

    while !quit {
        board.render_board(&mut stdout);
        board.next();

        stdout.flush().unwrap();

        if let Ok(key) = rx.recv_timeout(timeout) {
            match key {
                Key::Ctrl('c') | Key::Char('q') => quit = true,
                _ => {}
            }
        }
    }

    write!(
        stdout,
        "{}{}{}",
        cursor::Goto(1, 1),
        clear::All,
        cursor::Show
    )
    .unwrap();
}