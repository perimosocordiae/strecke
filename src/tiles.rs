#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Direction {
    North,
    West,
    South,
    East,
}

impl Direction {
    fn turn_left(&self) -> Direction {
        match self {
            Direction::North => Direction::West,
            Direction::West => Direction::South,
            Direction::South => Direction::East,
            Direction::East => Direction::North,
        }
    }
    pub fn grid_offsets(&self) -> (i8, i8) {
        match self {
            Direction::North => (-1, 0),
            Direction::West => (0, -1),
            Direction::South => (1, 0),
            Direction::East => (0, 1),
        }
    }
}

#[test]
fn test_turn_left() {
    let mut r = Direction::South.turn_left();
    assert_eq!(r, Direction::East);
    r = r.turn_left();
    assert_eq!(r, Direction::North);
    r = r.turn_left();
    assert_eq!(r, Direction::West);
    r = r.turn_left();
    assert_eq!(r, Direction::South);
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Port {
    A, // Top left
    B, // Top right
    C, // Right top
    D, // Right bottom
    E, // Bottom right
    F, // Bottom left
    G, // Left bottom
    H, // Left top
}

impl Port {
    fn turn_left(&self) -> Self {
        match self {
            Port::A => Port::C,
            Port::B => Port::D,
            Port::C => Port::E,
            Port::D => Port::F,
            Port::E => Port::G,
            Port::F => Port::H,
            Port::G => Port::A,
            Port::H => Port::B,
        }
    }
    fn turn_right(&self) -> Self {
        match self {
            Port::A => Port::G,
            Port::B => Port::H,
            Port::C => Port::A,
            Port::D => Port::B,
            Port::E => Port::C,
            Port::F => Port::D,
            Port::G => Port::E,
            Port::H => Port::F,
        }
    }
    pub fn flip(&self) -> Self {
        match self {
            Port::A => Port::F,
            Port::B => Port::E,
            Port::C => Port::H,
            Port::D => Port::G,
            Port::E => Port::B,
            Port::F => Port::A,
            Port::G => Port::D,
            Port::H => Port::C,
        }
    }
    pub fn facing_side(&self) -> Direction {
        match self {
            Port::A | Port::B => Direction::North,
            Port::C | Port::D => Direction::East,
            Port::E | Port::F => Direction::South,
            Port::G | Port::H => Direction::West,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Tile {
    layout: [(Port, Port); 4],
    facing: Direction,
}

impl Tile {
    pub fn rotate_left(mut self) -> Self {
        self.facing = self.facing.turn_left();
        self
    }
    fn normalize_port(&self, p: Port) -> Port {
        match self.facing {
            Direction::North => p,
            Direction::South => p.flip(),
            Direction::East => p.turn_left(),
            Direction::West => p.turn_right(),
        }
    }
    fn unnormalize_port(&self, p: Port) -> Port {
        match self.facing {
            Direction::North => p,
            Direction::South => p.flip(),
            Direction::East => p.turn_right(),
            Direction::West => p.turn_left(),
        }
    }
    pub fn traverse(&self, from: Port) -> Port {
        let start = self.normalize_port(from.flip());
        for (p1, p2) in self.layout.iter() {
            if start == *p1 {
                return self.unnormalize_port(*p2);
            } else if start == *p2 {
                return self.unnormalize_port(*p1);
            }
        }
        panic!("Unreachable path: start={:?}, tile={:?}", start, self);
    }
}

pub fn all_tiles() -> Vec<Tile> {
    vec![
        Tile {
            layout: [
                (Port::A, Port::E),
                (Port::B, Port::F),
                (Port::C, Port::H),
                (Port::D, Port::G),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::E),
                (Port::B, Port::F),
                (Port::C, Port::G),
                (Port::D, Port::H),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::F),
                (Port::B, Port::E),
                (Port::C, Port::H),
                (Port::D, Port::G),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::E),
                (Port::B, Port::D),
                (Port::C, Port::G),
                (Port::F, Port::H),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::H),
                (Port::B, Port::C),
                (Port::D, Port::H),
                (Port::F, Port::G),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::E),
                (Port::B, Port::C),
                (Port::D, Port::H),
                (Port::F, Port::G),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::E),
                (Port::B, Port::C),
                (Port::D, Port::G),
                (Port::F, Port::H),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::D),
                (Port::B, Port::G),
                (Port::C, Port::F),
                (Port::E, Port::H),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::D),
                (Port::B, Port::F),
                (Port::C, Port::G),
                (Port::E, Port::H),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::D),
                (Port::B, Port::E),
                (Port::C, Port::H),
                (Port::F, Port::G),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::D),
                (Port::B, Port::E),
                (Port::C, Port::G),
                (Port::F, Port::H),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::D),
                (Port::B, Port::C),
                (Port::E, Port::H),
                (Port::F, Port::G),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::C),
                (Port::B, Port::H),
                (Port::D, Port::F),
                (Port::E, Port::G),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::C),
                (Port::B, Port::H),
                (Port::D, Port::E),
                (Port::F, Port::G),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::C),
                (Port::B, Port::G),
                (Port::D, Port::F),
                (Port::E, Port::H),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::C),
                (Port::B, Port::G),
                (Port::D, Port::E),
                (Port::F, Port::H),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::C),
                (Port::B, Port::F),
                (Port::D, Port::H),
                (Port::E, Port::G),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::C),
                (Port::B, Port::F),
                (Port::D, Port::G),
                (Port::E, Port::H),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::C),
                (Port::B, Port::E),
                (Port::D, Port::H),
                (Port::F, Port::G),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::C),
                (Port::B, Port::E),
                (Port::D, Port::G),
                (Port::F, Port::H),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::C),
                (Port::B, Port::D),
                (Port::E, Port::H),
                (Port::F, Port::G),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::C),
                (Port::B, Port::D),
                (Port::E, Port::G),
                (Port::F, Port::H),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::B),
                (Port::C, Port::H),
                (Port::D, Port::G),
                (Port::E, Port::F),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::B),
                (Port::C, Port::H),
                (Port::D, Port::F),
                (Port::E, Port::G),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::B),
                (Port::C, Port::H),
                (Port::D, Port::E),
                (Port::F, Port::G),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::B),
                (Port::C, Port::G),
                (Port::D, Port::H),
                (Port::E, Port::F),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::B),
                (Port::C, Port::G),
                (Port::D, Port::F),
                (Port::E, Port::H),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::B),
                (Port::C, Port::G),
                (Port::D, Port::E),
                (Port::F, Port::H),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::B),
                (Port::C, Port::F),
                (Port::D, Port::H),
                (Port::E, Port::G),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::B),
                (Port::C, Port::F),
                (Port::D, Port::G),
                (Port::E, Port::H),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::B),
                (Port::C, Port::E),
                (Port::D, Port::H),
                (Port::F, Port::G),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::B),
                (Port::C, Port::E),
                (Port::D, Port::G),
                (Port::F, Port::H),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::B),
                (Port::C, Port::D),
                (Port::E, Port::H),
                (Port::F, Port::G),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::B),
                (Port::C, Port::D),
                (Port::E, Port::G),
                (Port::F, Port::H),
            ],
            facing: Direction::North,
        },
        Tile {
            layout: [
                (Port::A, Port::B),
                (Port::C, Port::D),
                (Port::E, Port::F),
                (Port::G, Port::H),
            ],
            facing: Direction::North,
        },
    ]
}

#[test]
fn test_all_tiles() {
    assert_eq!(all_tiles().len(), 35);
}
