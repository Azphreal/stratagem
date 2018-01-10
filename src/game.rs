use std::io::{self, Read, Write};
use termion::{clear, cursor};
use termion::color as termcol;
use termion::event::Key;
use termion::input::TermRead;

use board::{self, Board, Coord, Move, Tile};
use error;

const BOARD_WIDTH: u16 = 32;
const BOARD_HEIGHT: u16 = 12;
const SLEEP_DURATION: u64 = 500;

enum GameResult {
    RedWin,
    RedLoss,
    Draw,
    Ongoing
}

struct Game<R, W: Write> {
    board:       Board,
    cursor:      Coord,
    sel:         Option<Coord>,
    highlighted: Vec<Coord>,
    size:        (u16, u16),
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

pub fn init<R: Read, W: Write>(stdin: R, mut stdout: W, size: (u16, u16)) -> error::Result<()> {
    write!(stdout, "{}", clear::All)?;

    let mut game = Game {
        board:       Board::new(),
        cursor:      Coord { x: 0, y: 9 },
        sel:         None,
        highlighted: vec![],
        size:        size,
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
    pub fn run(&mut self) -> error::Result<()> {
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
                Char(' ') | Char('\n') => {
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
                                player = player.other();
                            }

                            self.sel = None;
                            self.highlighted.clear();
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
    fn setup(&mut self, player: board::Colour) -> error::Result<()> {
        use board::Piece::*;
        let mut to_place = vec![
            Flag, Bomb, Bomb, Bomb, Bomb, Bomb, Bomb, Marshall, General,
            Colonel, Colonel, Major, Major, Major, Captain, Captain, Captain,
            Captain, Lieutenant, Lieutenant, Lieutenant, Lieutenant, Sergeant,
            Sergeant, Sergeant, Sergeant, Miner, Miner, Miner, Miner, Miner,
            Scout, Scout, Scout, Scout, Scout, Scout, Scout, Scout, Spy
        ];

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
            }
        }

        self.draw_status(
            format!("Next to place: {}", to_place[0])
        )?;
        self.refresh(player)?;

        while let Ok(k) = self.stdin.next().unwrap() {
            use termion::event::Key::*;

            match k {
                Char('w') | Up    => self.cursor = mv!(0, -1),
                Char('a') | Left  => self.cursor = mv!(-1, 0),
                Char('s') | Down  => self.cursor = mv!(0, 1),
                Char('d') | Right => self.cursor = mv!(1, 0),
                Char('q') => return Err(error::Error::EarlyExit),
                Char('e') => {
                    let last = to_place[0];
                    to_place.push(last);
                    to_place.remove(0);
                }
                Char(' ') | Char('\n') => {
                    if self.highlighted.contains(&self.cursor) {
                        let piece = to_place[0];
                        let tile = Tile::Piece(piece, player);
                        self.board.set_tile(self.cursor, tile);
                        self.highlighted.remove_item(&self.cursor);
                        to_place.remove(0);
                    }
                }
                _ => {}
            }

            self.refresh(player)?;
            if to_place.is_empty() {
                break
            } else {
                self.draw_status(
                    format!("Next to place: {}", to_place[0])
                )?;
                self.stdout.flush()?;
            }
        }

        Ok(())
    }

    fn refresh(&mut self, player: board::Colour) -> error::Result<()> {
        self.draw_board(player)?;
        self.highlight()?;
        self.draw_cursor(player)?;
        self.stdout.flush()?;
        Ok(())
    }

    fn term_coords(&self, c: Coord) -> (u16, u16) {
        let tl = self.top_left();
        ((c.x + 1) * 3 + tl.0, c.y + 2 + tl.1)
    }

    fn draw_status<D>(&mut self, status: D) -> error::Result<()>
        where D: ::std::fmt::Display
    {
        let tl = self.top_left();
        write!(self.stdout,
               "{}{}",
               cursor::Goto(tl.0 + 1, tl.1 + 1 + BOARD_HEIGHT),
               status
        )?;
        Ok(())
    }

    fn top_left(&self) -> (u16, u16) {
        ((self.size.0 - BOARD_WIDTH) / 2, (self.size.1 - BOARD_HEIGHT) / 2)
    }

    fn draw_board(&mut self, player: board::Colour) -> error::Result<()> {
        let tl = self.top_left();

        for (n, line) in self.board
            .display_to(player)
            .unwrap()
            .split('\n')
            .enumerate()
        {
            write!(self.stdout,
                   "{}{}{}",
                   cursor::Goto(1 + tl.0, 1 + tl.1 + n as u16),
                   cursor::Hide,
                   // self.size,
                   line
            )?
        }
        Ok(())
    }

    fn draw_cursor(&mut self, player: board::Colour) -> error::Result<()> {
        let (x, y) = self.term_coords(self.cursor);
        let cursor =
            format!("[{}]", self.board.tile_at(self.cursor).show(player));

        write!(self.stdout, "{}", cursor::Goto(x - 1, y))?;
        if self.highlighted.contains(&self.cursor) {
            write!(self.stdout,
                   "{}{}{}",
                   termcol::Bg(termcol::Red),
                   cursor,
                   termcol::Bg(termcol::Reset)
            )?;
        } else {
            write!(self.stdout, "{}", cursor)?;
        };

        Ok(())
    }

    fn highlight(&mut self) -> error::Result<()> {
        for t in &self.highlighted {
            let (x, y) = self.term_coords(t.clone());
            write!(self.stdout, "{}{}   {}",
                   cursor::Goto(x - 1, y),
                   termcol::Bg(termcol::Red),
                   termcol::Bg(termcol::Reset)
            )?;
        }

        Ok(())
    }

    pub fn reveal(&mut self, c: Coord, player: board::Colour) -> error::Result<()> {
        if let Tile::Piece(p, col) = self.board.tile_at(c) {
            self.board.set_tile(c, Tile::Piece(p, col.other()));
            self.refresh(player)?;
            ::std::thread::sleep(::std::time::Duration::from_millis(SLEEP_DURATION));

            self.board.set_tile(c, Tile::Piece(p, col));
            self.refresh(player)?;
        }

        Ok(())
    }

    fn popup(&mut self, text: &str) -> error::Result<()> {
        Ok(())
    }

    fn check_game_end(&mut self) -> GameResult {
        let mut red = vec![];
        let mut blue = vec![];

        for line in &self.board {
            for tile in line {
                if let &Tile::Piece(piece, col) = tile {
                    match col {
                        board::Colour::Red => red.push(piece),
                        board::Colour::Blue => blue.push(piece),
                    }
                }
            }
        }

        // Flags
        if !red.contains(&board::Piece::Flag) {
            return GameResult::RedLoss
        } else if !blue.contains(&board::Piece::Flag) {
            return GameResult::RedWin
        }

        // Only immobile units
        if red.iter().fold(true, |acc, &p| acc && (p == board::Piece::Flag || p == board::Piece::Bomb)) {
            return GameResult::RedLoss
        } else if blue.iter().fold(true, |acc, &p| acc && (p == board::Piece::Flag || p == board::Piece::Bomb)) {
            return GameResult::RedWin
        }

        GameResult::Ongoing
    }
}
