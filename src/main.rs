use std::io::{stdin, stdout, Write};
use std::sync::mpsc;
use std::time::Duration;
use std::{fmt, thread};

use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, cursor};

const APP_NAME: &str = "GoLrs";
const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
const BIN_NAME: Option<&str> = option_env!("CARGO_PKG_NAME");
const DESCRIPTION: Option<&str> = option_env!("CARGO_PKG_DESCRIPTION");
const AUTHORS: Option<&str> = option_env!("CARGO_PKG_AUTHORS");

const HELP_TEMPLATE: &str = "\
GoLrs ({version}) - {about-with-newline}
{usage-heading} {usage}
{all-args}
{author-section}";

const MAX_ROWS: u16 = 125;
const MAX_COLS: u16 = 125;
const MIN_ROWS: u16 = 10;
const MIN_COLS: u16 = 10;
const DEFAULT_ROWS: u16 = 20;
const DEFAULT_COLS: u16 = 20;

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
        let rows = rows.clamp(MIN_ROWS as usize, MAX_ROWS as usize);
        let cols = cols.clamp(MIN_COLS as usize, MAX_COLS as usize);

        let mut board = vec![vec![Cell::Dead; cols]; rows];
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

        n
    }

    fn next(&mut self) {
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

        write!(s, "{}", cursor::Goto(1, self.rows as u16 + 1)).unwrap();
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
    let matches = get_args();
    let cols = matches.get_one::<u16>("columns").unwrap_or(&DEFAULT_COLS);
    let rows = matches.get_one::<u16>("rows").unwrap_or(&DEFAULT_ROWS);

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
    let mut board = Board::new(*rows as usize, *cols as usize);

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

fn get_args() -> ArgMatches {
    Command::new(APP_NAME)
        .display_name(BIN_NAME.unwrap_or("Unknown"))
        .author(AUTHORS.unwrap_or("Unknown"))
        .about(DESCRIPTION.unwrap_or("Unknown"))
        .version(VERSION.unwrap_or("Unknown"))
        .help_template(HELP_TEMPLATE)
        .arg(
            Arg::new("columns")
                .short('c')
                .long("cols")
                .value_name("num")
                .action(ArgAction::Set)
                .help("Number of columns in the board")
                .value_parser(value_parser!(u16).range((MIN_COLS as i64)..=(MAX_COLS as i64))),
        )
        .arg(
            Arg::new("rows")
                .short('r')
                .long("rows")
                .value_name("num")
                .action(ArgAction::Set)
                .help("Number of rows in the board")
                .value_parser(value_parser!(u16).range((MIN_ROWS as i64)..=(MAX_ROWS as i64))),
        )
        .get_matches()
}
