use std::io::{self, Read, Write};
use termion::{clear, cursor};
use termion::event::Key;
use termion::input::TermRead;

use board::{self, Board, Coord, Move, Tile};

const X_OFFSET: u16 = 4;
const Y_OFFSET: u16 = 4;
const SLEEP_DURATION: u64 = 500;

struct Game<R, W: Write> {
    board:       Board,
    cursor:      Coord,
    sel:         Option<Coord>,
    highlighted: Vec<Coord>,
    stdin:       R,
    stdout:      W
}

impl<R, W: Write> Drop for Game<R, W> {
    fn drop(&mut self) {
        write!(
            self.stdout,
            "{}{}{}",
            clear::All,
            cursor::Show,
            cursor::Goto(1, 1)
        ).unwrap()
    }
}

pub fn init<R: Read, W: Write>(stdin: R, mut stdout: W) -> io::Result<()> {
    write!(stdout, "{}", clear::All)?;

    let mut game = Game {
        board:       Board::new(),
        cursor:      Coord { x: 0, y: 9 },
        sel:         None,
        highlighted: vec![],
        stdin:       stdin.keys(),
        stdout:      stdout,
    };

    game.setup(board::Colour::Red)?;
    game.board.randomise(board::Colour::Blue);
    // game.board.randomise(board::Colour::Red);
    game.refresh(board::Colour::Red)?;

    game.run()
}

impl<R: Iterator<Item = Result<Key, io::Error>>, W: Write> Game<R, W> {

    /// The main game loop.
    pub fn run(&mut self) -> io::Result<()> {
        let mut player = board::Colour::Red;

        macro_rules! mv {
            ($x:expr, $y:expr) => (match self.cursor.offset($x, $y) {
                Some(c) => c,
                None => self.cursor
            });
        }

        while let Ok(k) = self.stdin.next().unwrap() {
            use termion::event::Key::*;

            match k {
                Char('w') | Up    => self.cursor = mv!(0, -1),
                Char('a') | Left  => self.cursor = mv!(-1, 0),
                Char('s') | Down  => self.cursor = mv!(0, 1),
                Char('d') | Right => self.cursor = mv!(1, 0),
                Char('q') => break,
                Char(' ') => {
                    match self.sel {
                        Some(selected) => {
                            if self.highlighted.contains(&self.cursor) {

                                // Conduct the move.
                                match self.board.tile_at(self.cursor) {
                                    // Show the piece attempting to be taken,
                                    // then conduct the results.
                                    Tile::Piece(p_enemy, _) => {
                                        use board::BattleResult::*;

                                        if let Tile::Piece(p_owned, _) =
                                            self.board.tile_at(selected)
                                        {
                                            match p_owned.attack(p_enemy) {
                                                // FIXME Probably some way
                                                // of doing this without
                                                // reallocation, even if
                                                // it's just making `reveal`
                                                // take a mutable.
                                                Victory => {
                                                    let cur = self.cursor;
                                                    self.reveal(cur, player)?;
                                                    self.board.apply_move(Move::new(selected, self.cursor));
                                                },
                                                Loss => {
                                                    let cur = self.cursor;
                                                    self.reveal(cur, player)?;
                                                    self.board.set_tile(selected, Tile::Empty);
                                                }
                                                Draw => {
                                                    let cur = self.cursor;
                                                    self.reveal(cur, player)?;
                                                    self.board.set_tile(selected, Tile::Empty);
                                                    self.board.set_tile(self.cursor, Tile::Empty);
                                                }
                                            }
                                        }
                                    }

                                    // Else just move.
                                    _ => self.board
                                        .apply_move(Move {
                                            from: selected,
                                            to: self.cursor
                                        }),
                                }
                            }

                            self.sel = None;
                            self.highlighted.clear();
                            player = player.other();
                        }
                        None => {
                            if let Tile::Piece(_, col) = self.board.tile_at(self.cursor) {
                                if player != col {
                                    continue
                                }
                            }

                            // Highlight valid spaces
                            let moves = self.board.find_moves(self.cursor);
                            let coords =
                                moves.iter().map(|m| m.to).collect::<Vec<_>>();
                            if !coords.is_empty() {
                                self.highlighted = coords;
                                self.sel = Some(self.cursor);
                            }
                        }
                    }
                }
                _ => ()
            }

            self.refresh(player)?;
        }

        Ok(())
    }

    /// Prompts the user to set up their side of the board.
    ///
    /// By default, places pieces in order valued highest to lowest, with
    /// stationary pieces first (i.e., flag, bombs, marshall, general, ...).
    fn setup(&mut self, player: board::Colour) -> io::Result<()> {
        use board::Piece::*;
        let mut to_place = vec![
            Flag, Bomb, Bomb, Bomb, Bomb, Bomb, Bomb, Marshall, General,
            Colonel, Colonel, Major, Major, Major, Captain, Captain, Captain,
            Captain, Lieutenant, Lieutenant, Lieutenant, Lieutenant, Sergeant,
            Sergeant, Sergeant, Sergeant, Miner, Miner, Miner, Miner, Miner,
            Scout, Scout, Scout, Scout, Scout, Scout, Scout, Scout, Spy
        ];
        to_place.reverse();

        macro_rules! mv {
            ($x:expr, $y:expr) => (match self.cursor.offset($x, $y) {
                Some(c) => c,
                None => self.cursor
            });
        }

        let offset = match player {
            board::Colour::Red => 6,
            board::Colour::Blue => 0,
        };

        for x in 0 .. 10 {
            for y in 0 .. 4 {
                let coord = Coord {x: x, y: y + offset};
                self.highlighted.push(coord);
                // let piece =
                //     to_place.pop().expect("Unexpected end of placement list");
                // let tile = Tile::Piece(piece, player);
                // self.set_tile(coord, tile);
            }
        }

        self.refresh(player)?;

        while let Ok(k) = self.stdin.next().unwrap() {
            use termion::event::Key::*;

            match k {
                Char('w') | Up    => self.cursor = mv!(0, -1),
                Char('a') | Left  => self.cursor = mv!(-1, 0),
                Char('s') | Down  => self.cursor = mv!(0, 1),
                Char('d') | Right => self.cursor = mv!(1, 0),
                Char('q') => break,
                Char(' ') => {
                    if self.highlighted.contains(&self.cursor) {
                        let piece =
                            to_place.pop().expect("Unexpected end of placement list");
                        let tile = Tile::Piece(piece, player);
                        self.board.set_tile(self.cursor, tile);
                        self.highlighted.remove_item(&self.cursor);
                    }
                }
                _ => {}
            }

            // to_place.clear();

            self.refresh(player)?;
            if to_place.is_empty() { break }
        }

        Ok(())
    }

    fn refresh(&mut self, player: board::Colour) -> io::Result<()> {
        self.draw_board(player)?;
        self.highlight()?;
        self.draw_cursor(player)?;
        self.stdout.flush()?;
        Ok(())
    }

    fn term_coords(&self, c: Coord) -> (u16, u16) {
        (c.x * 3 + 2 + X_OFFSET, c.y + 1 + Y_OFFSET)
    }

    fn draw_board(&mut self, player: board::Colour) -> io::Result<()> {
        for (n, line) in self.board
            .display_to(player)
            .unwrap()
            .split('\n')
            .enumerate()
        {
            write!(self.stdout,
                   "{}{}{}",
                   cursor::Goto(X_OFFSET, Y_OFFSET + n as u16),
                   cursor::Hide,
                   line
            )?
        }
        Ok(())
    }

    fn draw_cursor(&mut self, player: board::Colour) -> io::Result<()> {
        let (x, y) = self.term_coords(self.cursor);
        let cursor =
            format!("[{}]", self.board.tile_at(self.cursor).show(player));

        write!(self.stdout, "{}", cursor::Goto(x - 1, y))?;
        if self.highlighted.contains(&self.cursor) {
            use termion::color;
            write!(self.stdout,
                   "{}{}{}",
                   color::Bg(color::Red),
                   cursor,
                   color::Bg(color::Reset)
            )?;
        } else {
            write!(self.stdout, "{}", cursor)?;
        };

        Ok(())
    }

    fn highlight(&mut self) -> io::Result<()> {
        use termion::color;

        for t in &self.highlighted {
            let (x, y) = self.term_coords(t.clone());
            write!(self.stdout, "{}{}   {}",
                   cursor::Goto(x - 1, y),
                   color::Bg(color::Red),
                   color::Bg(color::Reset)
            )?;
        }

        Ok(())
    }

    pub fn reveal(&mut self, c: Coord, player: board::Colour) -> io::Result<()> {
        if let Tile::Piece(p, col) = self.board.tile_at(c) {
            self.board.set_tile(c, Tile::Piece(p, col.other()));
            self.refresh(player)?;
            ::std::thread::sleep(::std::time::Duration::from_millis(SLEEP_DURATION));

            self.board.set_tile(c, Tile::Piece(p, col));
            self.refresh(player)?;
        }

        Ok(())
    }
}
