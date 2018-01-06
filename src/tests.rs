#[allow(unused_imports)]
use board::*;

#[test]
fn test_move_basic() {
    let mut board = Board::new();
    let e0 = Coord::from("e0").unwrap();
    board.set_tile(e0, Tile::Piece(Piece::Marshall, Colour::Red));

    let moves = board.find_moves(e0);
    let expected = vec![
        Move::new(e0, Coord::from("f0").unwrap()),
        Move::new(e0, Coord::from("d0").unwrap()),
        Move::new(e0, Coord::from("e1").unwrap()),
    ];

    assert_eq!(moves, expected)
}

#[test]
fn test_move_into_terrain() {
    let mut board = Board::new();
    let e1 = Coord::from("e1").unwrap();
    board.set_tile(e1, Tile::Piece(Piece::Marshall, Colour::Red));

    let moves = board.find_moves(e1);
    let expected = vec![
        Move::new(e1, Coord::from("f1").unwrap()),
        Move::new(e1, Coord::from("d1").unwrap()),
        Move::new(e1, Coord::from("e0").unwrap()),
    ];

    assert_eq!(moves, expected)
}

#[test]
fn test_move_into_enemy() {
    let mut board = Board::new();
    let e0 = Coord::from("e0").unwrap();
    board.set_tile(e0, Tile::Piece(Piece::Marshall, Colour::Red));
    board.set_tile(
        Coord::from("e1").unwrap(),
        Tile::Piece(Piece::Flag, Colour::Blue),
    );

    let moves = board.find_moves(e0);
    let expected = vec![
        Move::new(e0, Coord::from("f0").unwrap()),
        Move::new(e0, Coord::from("d0").unwrap()),
        Move::new(e0, Coord::from("e1").unwrap()),
    ];

    assert_eq!(moves, expected)
}

#[test]
fn test_move_bomb() {
    let mut board = Board::new();
    let e0 = Coord::from("e0").unwrap();
    board.set_tile(e0, Tile::Piece(Piece::Bomb, Colour::Red));

    let moves = board.find_moves(e0);

    assert_eq!(moves, vec![])
}

#[test]
fn test_move_scout() {
    let mut board = Board::new();
    let e0 = Coord::from("e0").unwrap();

    board.set_tile(e0, Tile::Piece(Piece::Scout, Colour::Red));
    // Ally three spaces above; should stop before.
    board.set_tile(
        Coord::from("h0").unwrap(),
        Tile::Piece(Piece::Flag, Colour::Red),
    );
    // Enemy three spaces below; should capture.
    board.set_tile(
        Coord::from("b0").unwrap(),
        Tile::Piece(Piece::Flag, Colour::Blue),
    );
    // Terrain two spaces right; should stop before.

    let moves = board.find_moves(e0);
    let expected = vec![
        Move::new(e0, Coord::from("f0").unwrap()),
        Move::new(e0, Coord::from("g0").unwrap()),
        Move::new(e0, Coord::from("d0").unwrap()),
        Move::new(e0, Coord::from("c0").unwrap()),
        Move::new(e0, Coord::from("b0").unwrap()),
        Move::new(e0, Coord::from("e1").unwrap()),
    ];

    assert_eq!(moves, expected);
}
