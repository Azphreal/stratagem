extern crate rand;
extern crate termion;

mod board;
mod tests;
mod game;

use std::io::{self, Write};
use termion::raw::IntoRawMode;

fn main() {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let stdin = io::stdin();
    let stdin = stdin.lock();
    let stderr = io::stderr();
    let mut stderr = stderr.lock();

    let stdout = stdout.into_raw_mode().unwrap();
    match game::init(stdin, stdout) {
        Err(e) => println!("Fatal error: {}", e),
        Ok(()) => ()
    }
}
