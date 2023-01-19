mod gol;

use std::io::{stdin, stdout, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, cursor};

use gol::*;

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
