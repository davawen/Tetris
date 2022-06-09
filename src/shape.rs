use std::io::{Stdout, Write};
use derive_more::{Deref, DerefMut};

use rand::{thread_rng, Rng};
use termion::{
    cursor,
    raw::RawTerminal
};

#[derive(Debug, Clone, Copy, Default)]
pub struct Pos(pub i16, pub i16);

// Offset every pos by 4
impl Pos {
    pub fn term_pos(&self) -> Pos {
        Pos( self.0 * 2 + 1 + 4, self.1 + 1 + 4 )
    }

    pub fn goto(&self, stdout: &mut RawTerminal<Stdout>) -> std::io::Result<()> {
        let pos = self.term_pos();
        write!(stdout, "{}", cursor::Goto(pos.0.try_into().unwrap(), pos.1.try_into().unwrap()))
    }
}

#[derive(Default, Debug, Deref, DerefMut)]
pub struct ShapeRotation(pub [[u16; 4]; 4]);

pub type Shape = [ShapeRotation; 4];

#[non_exhaustive]
pub struct Shapes;

impl Shapes {
    pub const I: Shape = [
        ShapeRotation([
            [ 0, 1, 0, 0 ],
            [ 0, 1, 0, 0 ],
            [ 0, 1, 0, 0 ],
            [ 0, 1, 0, 0 ]
        ]),
        ShapeRotation([
            [ 0, 0, 0, 0 ],
            [ 1, 1, 1, 1 ],
            [ 0, 0, 0, 0 ],
            [ 0, 0, 0, 0 ]
        ]),
        ShapeRotation([
            [ 0, 0, 1, 0 ],
            [ 0, 0, 1, 0 ],
            [ 0, 0, 1, 0 ],
            [ 0, 0, 1, 0 ]
        ]),
        ShapeRotation([
            [ 0, 0, 0, 0 ],
            [ 0, 0, 0, 0 ],
            [ 1, 1, 1, 1 ],
            [ 0, 0, 0, 0 ]
        ])
    ];

    pub const T: Shape = [
        ShapeRotation([
            [ 0, 1, 0, 0 ],
            [ 1, 1, 1, 0 ],
            [ 0, 0, 0, 0 ],
            [ 0, 0, 0, 0 ]
        ]),
        ShapeRotation([
            [ 0, 1, 0, 0 ],
            [ 0, 1, 1, 0 ],
            [ 0, 1, 0, 0 ],
            [ 0, 0, 0, 0 ]
        ]),
        ShapeRotation([
            [ 0, 0, 0, 0 ],
            [ 1, 1, 1, 0 ],
            [ 0, 1, 0, 0 ],
            [ 0, 0, 0, 0 ]
        ]),
        ShapeRotation([
            [ 0, 1, 0, 0 ],
            [ 1, 1, 0, 0 ],
            [ 0, 1, 0, 0 ],
            [ 0, 0, 0, 0 ]
        ])
    ];

    pub const L: Shape = [
        ShapeRotation([
            [ 0, 1, 0, 0 ],
            [ 0, 1, 0, 0 ],
            [ 0, 1, 1, 0 ],
            [ 0, 0, 0, 0 ]
        ]),
        ShapeRotation([
            [ 0, 0, 0, 0 ],
            [ 1, 1, 1, 0 ],
            [ 1, 0, 0, 0 ],
            [ 0, 0, 0, 0 ]
        ]),
        ShapeRotation([
            [ 1, 1, 0, 0 ],
            [ 0, 1, 0, 0 ],
            [ 0, 1, 0, 0 ],
            [ 0, 0, 0, 0 ]
        ]),
        ShapeRotation([
            [ 0, 0, 1, 0 ],
            [ 1, 1, 1, 0 ],
            [ 0, 0, 0, 0 ],
            [ 0, 0, 0, 0 ]
        ])
    ];

    pub const J: Shape = [
        ShapeRotation([
            [ 0, 1, 0, 0 ],
            [ 0, 1, 0, 0 ],
            [ 1, 1, 0, 0 ],
            [ 0, 0, 0, 0 ]
        ]),
        ShapeRotation([
            [ 1, 0, 0, 0 ],
            [ 1, 1, 1, 0 ],
            [ 0, 0, 0, 0 ],
            [ 0, 0, 0, 0 ]
        ]),
        ShapeRotation([
            [ 0, 1, 1, 0 ],
            [ 0, 1, 0, 0 ],
            [ 0, 1, 0, 0 ],
            [ 0, 0, 0, 0 ]
        ]),
        ShapeRotation([
            [ 0, 0, 0, 0 ],
            [ 1, 1, 1, 0 ],
            [ 0, 0, 1, 0 ],
            [ 0, 0, 0, 0 ]
        ])
    ];

    pub fn rand() -> &'static Shape {
        match thread_rng().gen_range(1..=4) {
            1 => &Shapes::I,
            2 => &Shapes::T,
            3 => &Shapes::L,
            4 => &Shapes::J,
            _ => unreachable!()
        }
    }
}

