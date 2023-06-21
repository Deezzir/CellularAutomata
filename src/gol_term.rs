use std::io::{stdin, stdout, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, color, cursor, style};

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

    pub fn clear(&mut self) {
        self.board = Self::new(self.board.len(), self.board[0].len()).board;
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

#[derive(PartialEq)]
enum Mode {
    Run,
    Edit,
}

impl Mode {
    fn toggle(&mut self) {
        match self {
            Mode::Run => *self = Mode::Edit,
            Mode::Edit => *self = Mode::Run,
        }
    }
}

fn main() {
    let matches = get_args();
    let cols = matches.get_one::<u16>("columns").unwrap_or(&DEFAULT_COLS);
    let rows = matches.get_one::<u16>("rows").unwrap_or(&DEFAULT_ROWS);

    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", termion::cursor::Hide).unwrap();
    stdout.flush().unwrap();

    let timeout = Duration::from_millis(100);
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut keys = stdin().keys();
        while let Some(Ok(key)) = keys.next() {
            tx.send(key).unwrap();
        }
    });

    let mut quit = false;
    let mut mode = Mode::Edit;
    let mut board = Board::new(*rows as usize, *cols as usize);

    while !quit {
        match mode {
            Mode::Run => {
                board.to_unicode_mode();
                board.next_gen();
            }
            Mode::Edit => {
                board.to_ascii_mode();
            }
        }

        board.render(&mut stdout);
        stdout.flush().unwrap();

        if let Ok(key) = rx.recv_timeout(timeout) {
            match key {
                Key::Ctrl('c') | Key::Char('q') => quit = true,
                Key::Char('\n') => mode.toggle(),
                key => {
                    if mode == Mode::Edit {
                        match key {
                            Key::Char('c') => board.clear(),
                            Key::Char('r') => board.randomize(),
                            Key::Char('w') | Key::Up => board.move_cursor_up(),
                            Key::Char('s') | Key::Down => board.move_cursor_down(),
                            Key::Char('a') | Key::Left => board.move_cursor_left(),
                            Key::Char('d') | Key::Right => board.move_cursor_right(),
                            Key::Char(' ') => board.toggle_cur_cell(),
                            _ => {}
                        }
                    }
                }
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
