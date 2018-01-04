const DEFAULT_NO_MANS_LAND: [Tile; 10] =
    [Tile::Empty, Tile::Empty, Tile::Terrain, Tile::Terrain, Tile::Empty,
     Tile::Empty, Tile::Terrain, Tile::Terrain, Tile::Empty, Tile::Empty];

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Piece {
    Bomb,
    Marshall,
    General,
    Colonel,
    Major,
    Captain,
    Lieutenant,
    Sergeant,
    Miner,
    Scout,
    Spy,
    Flag
}

impl Piece {
    fn from(s: &str) -> Option<Piece> {
        match s {
            "bomb"       | "b" | "B" => Some(Piece::Bomb),
            "marshall"   | "1"       => Some(Piece::Marshall),
            "general"    | "2"       => Some(Piece::General),
            "colonel"    | "3"       => Some(Piece::Colonel),
            "major"      | "4"       => Some(Piece::Major),
            "captain"    | "5"       => Some(Piece::Captain),
            "lieutenant" | "6"       => Some(Piece::Lieutenant),
            "sergeant"   | "7"       => Some(Piece::Sergeant),
            "miner"      | "8"       => Some(Piece::Miner),
            "scout"      | "9"       => Some(Piece::Scout),
            "spy"        | "s" | "S" => Some(Piece::Spy),
            "flag"       | "f" | "F" => Some(Piece::Flag),
            _                  => None,
        }
    }
    fn value(&self) -> u8 {
        use self::Piece::*;
        match *self {
            Bomb       => ::std::u8::MAX,
            Marshall   => 10,
            General    => 9,
            Colonel    => 8,
            Major      => 7,
            Captain    => 6,
            Lieutenant => 5,
            Sergeant   => 4,
            Miner      => 3,
            Scout      => 2,
            Spy        => 1,
            Flag       => 0,
        }
    }
}

impl ::std::cmp::PartialOrd for Piece {
    fn partial_cmp(&self, other: &Piece) -> Option<::std::cmp::Ordering> {
        Some(self.value().cmp(&other.value()))
    }
}

impl ::std::fmt::Display for Piece {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        use self::Piece::*;
        match *self {
            Bomb       => write!(f, "B"),
            Marshall   => write!(f, "1"),
            General    => write!(f, "2"),
            Colonel    => write!(f, "3"),
            Major      => write!(f, "4"),
            Captain    => write!(f, "5"),
            Lieutenant => write!(f, "6"),
            Sergeant   => write!(f, "7"),
            Miner      => write!(f, "8"),
            Scout      => write!(f, "9"),
            Spy        => write!(f, "S"),
            Flag       => write!(f, "F"),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Colour{Red, Blue}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Tile {
    Terrain,
    Empty,
    Piece(Piece, Colour)
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Coord {
    x: usize,
    y: usize,
}

impl Coord {
    pub fn from(s: &str) -> Option<Self> {
        if s.len() != 2 {
            None
        } else {
            let s = s.chars().collect::<Vec<_>>();
            let (x, y) = (s[0], s[1]);
            if x < 'a' || x > 'j' || y < '0' || y > '9' {
                None
            } else {
                Some(Coord{
                    x: x.to_digit(20).unwrap() as usize - 10,
                    y: y.to_digit(10).unwrap() as usize,
                })
            }
        }
    }

    fn offset(&self, x: isize, y: isize) -> Option<Self> {
        let (mx, my) = (self.x as isize + x, self.y as isize + y);
        if mx < 0 || my < 0 || mx > 9 || my > 9 {
            None
        } else {
            Some(Coord {x: mx as usize, y: my as usize})
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Move {
    from: Coord,
    to: Coord
}

impl Move {
    pub fn new(f: Coord, t: Coord) -> Self {
        Move {from: f, to: t}
    }
}

#[derive(Debug)]
pub struct Board {
    board: [[Tile; 10]; 10],
    moves: Vec<Move>
}

impl Board {
    pub fn new() -> Self {
        Board {
            board: [
                [Tile::Empty; 10],
                [Tile::Empty; 10],
                [Tile::Empty; 10],
                [Tile::Empty; 10],
                DEFAULT_NO_MANS_LAND,
                DEFAULT_NO_MANS_LAND,
                [Tile::Empty; 10],
                [Tile::Empty; 10],
                [Tile::Empty; 10],
                [Tile::Empty; 10],
            ],
        moves: vec![]}
    }

    pub fn tile_at(&self, c: Coord) -> Tile {
        self.board[c.y][c.x]
    }

    pub fn set_tile(&mut self, c: Coord, t: Tile) {
        self.board[c.y][c.x] = t;
    }

    /// Mutates the game state with the provided move.
    ///
    /// There is **no legality checking** in this function. Use `find_moves`
    /// to present a list of legal moves before mutating the game state.
    pub fn apply_move(&mut self, m: Move) {
        let _old = self.tile_at(m.from);
        self.set_tile(m.to, _old);
        self.set_tile(m.from, Tile::Empty);
        self.moves.push(m);
    }

    /// Finds all legal moves available from the coordinate.
    ///
    /// Notably, the list of legal moves will be empty in the cases where:
    /// - `c` is an empty or terrain tile; or
    /// - the piece on the tile has surrounded by terrain and allied pieces.
    ///
    /// The list may contain more than four moves if the tile contains a scout,
    /// as they may move any number of spaces in an unbroken line.
    pub fn find_moves(&self, c: Coord) -> Vec<Move> {
        let mut mvs = Vec::new();
        match self.tile_at(c) {
            Tile::Piece(curr_piece, curr_col) => {
                match curr_piece {
                    Piece::Bomb | Piece::Flag
                        => return mvs,
                    Piece::Scout => {
                        // Iterate through the neighbours.
                        for &(x, y) in [(1, 0), (-1, 0), (0, 1), (0, -1)].iter() {
                            if let Some(next_c) = c.offset(x, y) {
                                match self.tile_at(next_c) {
                                    Tile::Piece(_, next_col) => {
                                        if curr_col != next_col {
                                            mvs.push(Move::new(c, next_c));
                                        }
                                    },
                                    // Go as far as possible.
                                    Tile::Empty => {
                                        mvs.push(Move::new(c, next_c));
                                        let mut mult = 2;
                                        'EXT: while let Some(next_c) = c.offset(x * mult, y * mult) {
                                            match self.tile_at(next_c) {
                                                Tile::Piece(_, next_col) => {
                                                    if curr_col != next_col {
                                                        mvs.push(Move::new(c, next_c));
                                                    }
                                                    break 'EXT
                                                },
                                                Tile::Empty => mvs.push(Move::new(c, next_c)),
                                                Tile::Terrain => break 'EXT
                                            }
                                            mult += 1;
                                        }
                                    }
                                    _ => (),
                                }
                            }
                        }
                        return mvs
                    }
                    _ => {
                        // Iterate through the neighbours.
                        for &(x, y) in [(1, 0), (-1, 0), (0, 1), (0, -1)].iter() {
                            if let Some(next_c) = c.offset(x, y) {
                                match self.tile_at(next_c) {
                                    Tile::Piece(_, next_col) => {
                                        if curr_col != next_col {
                                            mvs.push(Move::new(c, next_c));
                                        }
                                    },
                                    Tile::Empty => mvs.push(Move::new(c, next_c)),
                                    _ => (),
                                }
                            }
                        }
                        return mvs
                    }
                }
            }
            _ => return mvs
        }
    }

    /// Returns a formatted game state.
    ///
    /// Will blank out pieces that the player provided doesn't own, as it is
    /// considered personal knowledge.
    pub fn display_to(&self, player: Colour) -> String {
        use ::std::fmt::Write;

        let mut s = String::new();
        write!(s, "┌──────────────────────────────┐\n");
        for y in 0..10 {
            write!(s, "│");
            for x in 0..10 {
                let tile = self.tile_at(Coord {x: x, y: y});
                match tile {
                    Tile::Terrain => {write!(s, " ~ " );},
                    Tile::Empty => {write!(s, "   " );},
                    Tile::Piece(p, c) => if player == c {
                        write!(s, " {} ", p);
                    } else {
                        write!(s, " ▇ ");
                    }
                }
            }
            write!(s, "│\n");
        }
        write!(s, "└──────────────────────────────┘");
        s
    }

    /// For the lazy.
    ///
    /// Randomises the placement of the starting pieces on the given side (where
    /// blue is the top half, and red is the bottom half)
    pub fn randomise(&mut self, player: Colour) {
        use rand::Rng;
        use self::Piece::*;

        let mut rng = ::rand::thread_rng();
        // FIXME Make this less obvious?
        let mut to_place = vec![Bomb, Bomb, Bomb, Bomb, Bomb, Bomb,
                                Marshall, General, Colonel, Colonel,
                                Major, Major, Major,
                                Captain, Captain, Captain, Captain,
                                Lieutenant, Lieutenant, Lieutenant, Lieutenant,
                                Sergeant, Sergeant, Sergeant, Sergeant,
                                Miner, Miner, Miner, Miner, Miner,
                                Scout, Scout, Scout, Scout,
                                Scout, Scout, Scout, Scout,
                                Spy, Flag];

        {
            let mut sl = to_place.as_mut_slice();
            rng.shuffle(&mut sl);
        }

        let offset = match player {
            Colour::Red => 6,
            Colour::Blue => 0,
        };

        for x in 0..10 {
            for y in 0..4 {
                let coord = Coord {x: x, y: y + offset};
                let piece = to_place.pop()
                    .expect("Unexpected end of placement list");
                let tile = Tile::Piece(piece, player);
                self.set_tile(coord, tile);
            }
        }
    }
}

fn create_side(colour: Colour) -> [[Tile; 10]; 4] {
    match colour {
        Colour::Red => {
            unimplemented!()
        },
        // Default board
        Colour::Blue => DEFAULT_BLUE_SIDE,
    }
}
const DEFAULT_BLUE_SIDE: [[Tile; 10]; 4] =
    [[Tile::Piece(Piece::Miner,      Colour::Blue),
      Tile::Piece(Piece::Captain,    Colour::Blue),
      Tile::Piece(Piece::Scout,      Colour::Blue),
      Tile::Piece(Piece::Lieutenant, Colour::Blue),
      Tile::Piece(Piece::Scout,      Colour::Blue),
      Tile::Piece(Piece::Miner,      Colour::Blue),
      Tile::Piece(Piece::Scout,      Colour::Blue),
      Tile::Piece(Piece::Sergeant,   Colour::Blue),
      Tile::Piece(Piece::Miner,      Colour::Blue),
      Tile::Piece(Piece::Captain,    Colour::Blue)],

     [Tile::Piece(Piece::Scout,      Colour::Blue),
      Tile::Piece(Piece::Miner,      Colour::Blue),
      Tile::Piece(Piece::Captain,    Colour::Blue),
      Tile::Piece(Piece::Major,      Colour::Blue),
      Tile::Piece(Piece::Sergeant,   Colour::Blue),
      Tile::Piece(Piece::Spy,        Colour::Blue),
      Tile::Piece(Piece::Flag,       Colour::Blue),
      Tile::Piece(Piece::Scout,      Colour::Blue),
      Tile::Piece(Piece::Major,      Colour::Blue),
      Tile::Piece(Piece::Scout,      Colour::Blue)],

     [Tile::Piece(Piece::Scout,      Colour::Blue),
      Tile::Piece(Piece::Colonel,    Colour::Blue),
      Tile::Piece(Piece::Bomb,       Colour::Blue),
      Tile::Piece(Piece::Sergeant,   Colour::Blue),
      Tile::Piece(Piece::Bomb,       Colour::Blue),
      Tile::Piece(Piece::General,    Colour::Blue),
      Tile::Piece(Piece::Bomb,       Colour::Blue),
      Tile::Piece(Piece::Bomb,       Colour::Blue),
      Tile::Piece(Piece::Miner,      Colour::Blue),
      Tile::Piece(Piece::Lieutenant, Colour::Blue)],

     [Tile::Piece(Piece::Lieutenant, Colour::Blue),
      Tile::Piece(Piece::Scout,      Colour::Blue),
      Tile::Piece(Piece::Bomb,       Colour::Blue),
      Tile::Piece(Piece::Marshall,   Colour::Blue),
      Tile::Piece(Piece::Lieutenant, Colour::Blue),
      Tile::Piece(Piece::Captain,    Colour::Blue),
      Tile::Piece(Piece::Major,      Colour::Blue),
      Tile::Piece(Piece::Bomb,       Colour::Blue),
      Tile::Piece(Piece::Colonel,    Colour::Blue),
      Tile::Piece(Piece::Sergeant,   Colour::Blue)]];
