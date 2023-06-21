use rand::Rng;
use std::io::{stdin, stdout, Write};
use std::sync::mpsc;
use std::{f32::consts::PI, thread, time::Duration};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, cursor};

const WIDTH: usize = 150;
const HEIGHT: usize = 150;

const LEVEL: [char; 10] = [' ', '.', '-', '=', 'c', 'o', 'a', 'A', '@', '#'];
const RA: f32 = 21.0;
const RI: f32 = RA / 3.0;
const ALPHA_N: f32 = 0.028;
const ALPHA_M: f32 = 0.147;
const B1: f32 = 0.278;
const B2: f32 = 0.365;
const D1: f32 = 0.267;
const D2: f32 = 0.445;
const DT: f32 = 0.05;

struct Board {
    cells: Vec<Vec<f32>>,
}

impl Board {
    fn new(h: usize, w: usize) -> Board {
        Board {
            cells: vec![vec![0.0; w]; h],
        }
    }

    fn randomize(&mut self) {
        let mut rng = rand::thread_rng();
        for row in self.cells.iter_mut() {
            for cell in row.iter_mut() {
                *cell = rng.gen();
            }
        }
    }

    fn display<W: Write>(&self, s: &mut W) {
        write!(s, "{}{}", cursor::Goto(1, 1), clear::AfterCursor).unwrap();

        for (ir, row) in self.cells.iter().enumerate() {
            write!(s, "{}", cursor::Goto(1, ir as u16 + 1)).unwrap();

            for cell in row.iter() {
                let level_id = (cell * (LEVEL.len() - 1) as f32) as usize;
                write!(s, "{}", LEVEL[level_id]).unwrap();
                write!(s, "{}", LEVEL[level_id]).unwrap();
            }
            writeln!(s).unwrap();
        }
    }

    fn next(&mut self) {
        let cells = self.cells.clone();

        for cy in 0..cells.len() as i32 {
            for cx in 0..cells[0].len() as i32 {
                let mut m: f32 = 0.0;
                let mut n: f32 = 0.0;

                for dy in -(RA - 1.0) as i32..(RA - 1.0) as i32 {
                    for dx in -(RA - 1.0) as i32..(RA - 1.0) as i32 {
                        let x = emod(cx + dx, cells[0].len() as i32) as usize;
                        let y = emod(cy + dy, cells.len() as i32) as usize;

                        if dx * dx + dy * dy <= (RI * RI) as i32 {
                            m += cells[y][x];
                        } else if dx * dx + dy * dy <= (RA * RA) as i32 {
                            n += cells[y][x];
                        }
                    }
                }

                m /= PI * RI * RI;
                n /= PI * (RA * RA - RI * RI);

                let cell = self.cells[cy as usize][cx as usize] + DT * (2.0 * s(m, n) - 1.0);
                self.cells[cy as usize][cx as usize] = cell.clamp(0.0, 1.0);
            }
        }
    }
}

fn sigma(x: f32, a: f32, alpha: f32) -> f32 {
    1.0 / (1.0 + (-(x - a) * 4.0 / alpha).exp())
}

fn sigma_n(x: f32, a: f32, b: f32) -> f32 {
    sigma(x, a, ALPHA_N) * (1.0 - sigma(x, b, ALPHA_N))
}

fn sigma_m(x: f32, y: f32, m: f32) -> f32 {
    x * (1.0 - sigma(m, 0.5, ALPHA_M)) + y * sigma(m, 0.5, ALPHA_M)
}

fn s(m: f32, n: f32) -> f32 {
    sigma_n(n, sigma_m(B1, D1, m), sigma_m(B2, D2, m))
}

fn emod(a: i32, b: i32) -> i32 {
    (a % b + b) % b
}

fn main() {
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", termion::cursor::Hide).unwrap();
    stdout.flush().unwrap();

    let timeout = Duration::from_millis(25);
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut keys = stdin().keys();
        while let Some(Ok(key)) = keys.next() {
            tx.send(key).unwrap();
        }
    });

    let mut board = Board::new(HEIGHT, WIDTH);
    let mut quit = false;

    board.randomize();

    while !quit {
        board.display(&mut stdout);
        stdout.flush().unwrap();

        board.next();

        if let Ok(key) = rx.recv_timeout(timeout) {
            match key {
                Key::Ctrl('c') | Key::Char('q') => quit = true,
                _ => (),
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
