use std::io::{self, Read, Write};
use termion::{clear, cursor, style};
use termion::event::Key;
use termion::input::TermRead;

use board::{self, Board, Coord};

const X_OFFSET: u16 = 1;
const Y_OFFSET: u16 = 1;

struct Game<R, W: Write> {
    board: Board,
    sel: Coord,
    stdin: R,
    stdout: W,
}

impl<R, W: Write> Drop for Game<R, W> {
    fn drop(&mut self) {
        write!(self.stdout, "{}{}{}",
               clear::All, cursor::Show, cursor::Goto(1, 1)).unwrap()
    }
}

pub fn init<R: Read, W: Write>(stdin: R, mut stdout: W) {
    write!(stdout, "{}", clear::All).unwrap();

    let mut game = Game {
        board: Board::new(),
        sel: Coord {x: 0, y: 9},
        stdin: stdin.keys(),
        stdout: stdout,
    };

    game.setup(board::Colour::Red);

    // game.run();
}

impl<R: Iterator<Item=Result<Key, ::std::io::Error>>, W: Write> Game<R, W> {

    /// The main game loop.
    pub fn run(&mut self) {
        loop {

        }
    }

    /// Prompts the user to set up their side of the board.
    ///
    /// By default, places pieces in order valued highest to lowest, with
    /// stationary pieces first (i.e., flag, bombs, marshall, general, ...).
    fn setup(&mut self, player: board::Colour) {
        use board::Piece::*;
        let mut to_place = vec![Flag, Bomb, Bomb, Bomb, Bomb, Bomb, Bomb,
                                Marshall, General, Colonel, Colonel,
                                Major, Major, Major,
                                Captain, Captain, Captain, Captain,
                                Lieutenant, Lieutenant, Lieutenant, Lieutenant,
                                Sergeant, Sergeant, Sergeant, Sergeant,
                                Miner, Miner, Miner, Miner, Miner,
                                Scout, Scout, Scout, Scout,
                                Scout, Scout, Scout, Scout,
                                Spy];

        macro_rules! mv {
            ($x:expr, $y:expr) => (match self.sel.offset($x, $y) {
                Some(c) => c,
                None => self.sel
            });
        }

        self.board.randomise(player);
        self.refresh(player);

        while let Ok(k) = self.stdin.next().unwrap() {
            // let k = self.stdin.next().unwrap().unwrap();
            use termion::event::Key::*;

            match k {
                Char('w') | Up    => self.sel = mv!(0, -1),
                Char('a') | Left  => self.sel = mv!(-1, 0),
                Char('s') | Down  => self.sel = mv!(0, 1),
                Char('d') | Right => self.sel = mv!(1, 0),
                Char('q') => return,
                Char(' ') => {
                }
                _ => {},
            }

            self.refresh(player);
        }
    }

    fn refresh(&mut self, player: board::Colour) -> ::std::io::Result<()> {
        self.draw_board(player)?;
        self.draw_cursor()?;
        self.stdout.flush()?;
        Ok(())
    }

    fn draw_board(&mut self, player: board::Colour) -> ::std::io::Result<()> {
        let board = self.board.display_to(player).unwrap();
        for (n, line) in board.split('\n').enumerate() {
            write!(self.stdout, "{}{}{}",
                   cursor::Goto(X_OFFSET, Y_OFFSET + n as u16),
                   cursor::Hide,
                   line)?
        }
        Ok(())
    }

    fn draw_cursor(&mut self) -> ::std::io::Result<()> {
        write!(self.stdout, "{}{}{}{}",
               cursor::Goto(self.sel.x * 3 + 2, self.sel.y + 2),
               "[", self.board.tile_at(self.sel), "]")?;
        write!(self.stdout, "{}",
               cursor::Goto(self.sel.x * 3 + 3, self.sel.y + 2))?;
        Ok(())
    }

    fn highlight(&mut self, tiles: Vec<Coord>) {
        for t in tiles {
            unimplemented!()
        }
    }
}
