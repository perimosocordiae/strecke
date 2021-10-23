use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Serialize)]
pub enum Direction {
    North,
    West,
    South,
    East,
}

impl Direction {
    pub fn all() -> impl Iterator<Item = Direction> {
        [
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
        ]
        .iter()
        .copied()
    }
    pub fn grid_offsets(&self) -> (i8, i8) {
        match self {
            Direction::North => (-1, 0),
            Direction::West => (0, -1),
            Direction::South => (1, 0),
            Direction::East => (0, 1),
        }
    }
    fn normalize_port(&self, p: Port) -> Port {
        match self {
            Direction::North => p,
            Direction::South => p.turn_left().turn_left(),
            Direction::East => p.turn_left(),
            Direction::West => p.turn_right(),
        }
    }
    fn unnormalize_port(&self, p: Port) -> Port {
        match self {
            Direction::North => p,
            Direction::South => p.turn_left().turn_left(),
            Direction::East => p.turn_right(),
            Direction::West => p.turn_left(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Serialize)]
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
    fn turn_right(&self) -> Self {
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
    fn turn_left(&self) -> Self {
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

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Tile {
    layout: [(Port, Port); 4],
}

impl Tile {
    pub fn traverse(&self, from: Port, facing: Direction) -> Port {
        let start = facing.normalize_port(from);
        for (p1, p2) in self.layout.iter() {
            if start == *p1 {
                return facing.unnormalize_port(*p2);
            } else if start == *p2 {
                return facing.unnormalize_port(*p1);
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
        },
        Tile {
            layout: [
                (Port::A, Port::E),
                (Port::B, Port::F),
                (Port::C, Port::G),
                (Port::D, Port::H),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::F),
                (Port::B, Port::E),
                (Port::C, Port::H),
                (Port::D, Port::G),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::E),
                (Port::B, Port::D),
                (Port::C, Port::G),
                (Port::F, Port::H),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::H),
                (Port::B, Port::C),
                (Port::D, Port::E),
                (Port::F, Port::G),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::E),
                (Port::B, Port::C),
                (Port::D, Port::H),
                (Port::F, Port::G),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::E),
                (Port::B, Port::C),
                (Port::D, Port::G),
                (Port::F, Port::H),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::D),
                (Port::B, Port::G),
                (Port::C, Port::F),
                (Port::E, Port::H),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::D),
                (Port::B, Port::F),
                (Port::C, Port::G),
                (Port::E, Port::H),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::D),
                (Port::B, Port::E),
                (Port::C, Port::H),
                (Port::F, Port::G),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::D),
                (Port::B, Port::E),
                (Port::C, Port::G),
                (Port::F, Port::H),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::D),
                (Port::B, Port::C),
                (Port::E, Port::H),
                (Port::F, Port::G),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::C),
                (Port::B, Port::H),
                (Port::D, Port::F),
                (Port::E, Port::G),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::C),
                (Port::B, Port::H),
                (Port::D, Port::E),
                (Port::F, Port::G),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::C),
                (Port::B, Port::G),
                (Port::D, Port::F),
                (Port::E, Port::H),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::C),
                (Port::B, Port::G),
                (Port::D, Port::E),
                (Port::F, Port::H),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::C),
                (Port::B, Port::F),
                (Port::D, Port::H),
                (Port::E, Port::G),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::C),
                (Port::B, Port::F),
                (Port::D, Port::G),
                (Port::E, Port::H),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::C),
                (Port::B, Port::E),
                (Port::D, Port::H),
                (Port::F, Port::G),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::C),
                (Port::B, Port::E),
                (Port::D, Port::G),
                (Port::F, Port::H),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::C),
                (Port::B, Port::D),
                (Port::E, Port::H),
                (Port::F, Port::G),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::C),
                (Port::B, Port::D),
                (Port::E, Port::G),
                (Port::F, Port::H),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::B),
                (Port::C, Port::H),
                (Port::D, Port::G),
                (Port::E, Port::F),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::B),
                (Port::C, Port::H),
                (Port::D, Port::F),
                (Port::E, Port::G),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::B),
                (Port::C, Port::H),
                (Port::D, Port::E),
                (Port::F, Port::G),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::B),
                (Port::C, Port::G),
                (Port::D, Port::H),
                (Port::E, Port::F),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::B),
                (Port::C, Port::G),
                (Port::D, Port::F),
                (Port::E, Port::H),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::B),
                (Port::C, Port::G),
                (Port::D, Port::E),
                (Port::F, Port::H),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::B),
                (Port::C, Port::F),
                (Port::D, Port::H),
                (Port::E, Port::G),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::B),
                (Port::C, Port::F),
                (Port::D, Port::G),
                (Port::E, Port::H),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::B),
                (Port::C, Port::E),
                (Port::D, Port::H),
                (Port::F, Port::G),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::B),
                (Port::C, Port::E),
                (Port::D, Port::G),
                (Port::F, Port::H),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::B),
                (Port::C, Port::D),
                (Port::E, Port::H),
                (Port::F, Port::G),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::B),
                (Port::C, Port::D),
                (Port::E, Port::G),
                (Port::F, Port::H),
            ],
        },
        Tile {
            layout: [
                (Port::A, Port::B),
                (Port::C, Port::D),
                (Port::E, Port::F),
                (Port::G, Port::H),
            ],
        },
    ]
}

#[test]
fn test_all_tiles() {
    assert_eq!(all_tiles().len(), 35);
}
