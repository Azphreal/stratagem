#![feature(vec_remove_item)]
#[macro_use] extern crate failure;
extern crate rand;
extern crate termion;

mod board;
mod error;
mod tests;
mod game;

use std::io::{self, Write};
use termion::raw::IntoRawMode;

fn main() {
    let termsize = match termion::terminal_size() {
        Ok(s) => s,
        Err(_) => (40, 40),
    };
    if termsize.0 < 32 || termsize.1 < 12 {
        println!("Stratagem requires a minimum terminal size of 32 x 12.");
        println!("Enlarge your terminal and try again.");
        ::std::process::exit(1);
    }

    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let stdin = io::stdin();
    let stdin = stdin.lock();
    let stderr = io::stderr();
    let mut stderr = stderr.lock();

    let stdout = stdout.into_raw_mode().unwrap();

    match game::init(stdin, stdout, termsize).err() {
        Some(error::Error::EarlyExit) => (),
        Some(e) => {
            println!("Fatal error: {}", e);
            ::std::process::exit(1)
        }
        None => ()
    }
}
