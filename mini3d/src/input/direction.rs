#[derive(Clone, Copy)]
pub enum Direction {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

impl Direction {
    
    pub const COUNT: usize = 4;

    pub fn iterator() -> impl Iterator<Item = Direction> {
        [Direction::Up, Direction::Down, Direction::Left, Direction::Right].iter().copied()
    }
}

