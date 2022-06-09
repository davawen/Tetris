use rand::{prelude::*, distributions::Standard};

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Empty,
    White,
    Red,
    Cyan,
    Yellow,
    Green
}

impl Default for Color {
    fn default() -> Self {
        Color::Empty
    }
}

impl Color {
    pub fn is_empty(&self) -> bool {
        matches!(self, Color::Empty)
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use termion::color as c;
        use Color::*;

        match self {
            Empty => write!(f, "{}", c::Fg(c::Black)),
            White => write!(f, "{}", c::Fg(c::LightWhite)),
            Red => write!(f, "{}", c::Fg(c::Red)),
            Cyan => write!(f, "{}", c::Fg(c::Cyan)),
            Yellow => write!(f, "{}", c::Fg(c::Yellow)),
            Green => write!(f, "{}", c::Fg(c::LightGreen)),
        }
    }
}

impl Distribution<Color> for Standard {
    /// Doesn't return 'Empty'
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Color {
        use Color::*;

        match rng.gen_range(1..=5) {
            1 => White,
            2 => Red,
            3 => Cyan,
            4 => Yellow,
            5 => Green,
            _ => panic!()
        }
    }
}

