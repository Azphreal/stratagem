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
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let stdin = io::stdin();
    let stdin = stdin.lock();
    let stderr = io::stderr();
    let mut stderr = stderr.lock();

    let stdout = stdout.into_raw_mode().unwrap();
    match game::init(stdin, stdout).err() {
        Some(error::Error::EarlyExit) => (),
        Some(e) => {
            println!("Fatal error: {}", e);
            ::std::process::exit(1)
        }
        None => ()
    }
}
