use std::io::{Stdout, Write};
use derive_more::{Deref, DerefMut, Add, AddAssign, Neg};

use rand::{thread_rng, Rng};
use termion::{
    cursor,
    raw::RawTerminal
};

#[derive(Debug, Clone, Copy, Default, Add, AddAssign, Neg)]
pub struct Pos(pub i16, pub i16);

#[derive(Debug, Clone, Copy, Default, Add, AddAssign)]
pub struct Size(pub i16, pub i16);

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

impl Size {
    pub fn term_value(&self) -> Size {
        Size(self.0 * 2, self.1)
    }
}

#[derive(Default, Debug, Deref, DerefMut)]
pub struct ShapeRotation(pub [[u16; 4]; 4]);

pub type Shape = &'static [ShapeRotation];

#[non_exhaustive]
pub struct Shapes;

impl Shapes {
    pub const I: [ShapeRotation; 2] = [
        ShapeRotation([
            [ 0, 0, 1, 0 ],
            [ 0, 0, 1, 0 ],
            [ 0, 0, 1, 0 ],
            [ 0, 0, 1, 0 ]
        ]),
        ShapeRotation([
            [ 0, 0, 0, 0 ],
            [ 1, 1, 1, 1 ],
            [ 0, 0, 0, 0 ],
            [ 0, 0, 0, 0 ]
        ])
    ];

    pub const O: [ShapeRotation; 1] = [
        ShapeRotation([
            [ 0, 0, 0, 0 ],
            [ 0, 1, 1, 0 ],
            [ 0, 1, 1, 0 ],
            [ 0, 0, 0, 0 ]
        ])
    ];

    pub const T: [ShapeRotation; 4] = [
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

    pub const L: [ShapeRotation; 4] = [
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

    pub const J: [ShapeRotation; 4] = [
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

    pub const S: [ShapeRotation; 2] = [
        ShapeRotation([
            [ 0, 0, 0, 0 ],
            [ 0, 1, 1, 0 ],
            [ 1, 1, 0, 0 ],
            [ 0, 0, 0, 0 ]
        ]),
        ShapeRotation([
            [ 0, 1, 0, 0 ],
            [ 0, 1, 1, 0 ],
            [ 0, 0, 1, 0 ],
            [ 0, 0, 0, 0 ]
        ])
    ];

    pub const Z: [ShapeRotation; 2] = [
        ShapeRotation([
            [ 0, 0, 0, 0 ],
            [ 1, 1, 0, 0 ],
            [ 0, 1, 1, 0 ],
            [ 0, 0, 0, 0 ]
        ]),
        ShapeRotation([
            [ 0, 0, 1, 0 ],
            [ 0, 1, 1, 0 ],
            [ 0, 1, 0, 0 ],
            [ 0, 0, 0, 0 ]
        ]),
    ];

    pub fn rand() -> Shape {
        match thread_rng().gen_range(1..=7) {
            1 => &Shapes::I,
            2 => &Shapes::O,
            3 => &Shapes::T,
            4 => &Shapes::L,
            5 => &Shapes::J,
            6 => &Shapes::S,
            7 => &Shapes::Z,
            _ => unreachable!()
        }
    }
}
