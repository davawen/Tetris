use std::{io::{stdout, Write, stdin, Stdout}, thread, time::{Duration, Instant}, sync::{mpsc::{self, TryRecvError}}, default};
use derive_more::{Deref, DerefMut};

use rand::{thread_rng, prelude::Distribution, distributions::Standard, Rng};
use termion::{
    cursor,
    raw::{IntoRawMode, RawTerminal},
    input::TermRead,
    event::Key, color::Fg
};

#[derive(Debug, Clone, Copy)]
enum Color {
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
    fn is_empty(&self) -> bool {
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

#[derive(Debug, Clone, Copy, Default)]
struct Pos(i16, i16);

// Offset every pos by 4
impl Pos {
    fn term_pos(&self) -> Pos {
        Pos( self.0 * 2 + 1 + 4, self.1 + 1 + 4 )
    }

    fn goto(&self, stdout: &mut RawTerminal<Stdout>) -> std::io::Result<()> {
        let pos = self.term_pos();
        write!(stdout, "{}", cursor::Goto(pos.0.try_into().unwrap(), pos.1.try_into().unwrap()))
    }
}

#[derive(Default, Debug, Deref, DerefMut)]
struct ShapeRotation([[u16; 4]; 4]);

type Shape = [ShapeRotation; 4];

#[derive(Default, Debug, Deref, DerefMut)]
struct Grid([[Color; 8]; 16]);

#[derive(Debug)]
struct Piece {
    color: Color,
    shape: &'static Shape,
    pos: Pos,
    rotation: u8
}

#[non_exhaustive]
struct Shapes;

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

    fn rand() -> &'static Shape {
        match thread_rng().gen_range(1..=4) {
            1 => &Shapes::I,
            2 => &Shapes::T,
            3 => &Shapes::L,
            4 => &Shapes::J,
            _ => unreachable!()
        }
    }
}

impl Grid {
    const WIDTH: i16 = 8;
    const HEIGHT: i16 = 16;

    fn check_collision(&self, block: &Piece) -> bool {
        
        for y in 0..block.shape.len() {
            for x in 0..block.shape[y].len() {
                // Only check for actual blocks in the block
                if block.shape_rotation()[y][x] != 0 {

                    let (g_x, g_y) = (x as i16 + block.pos.0, y as i16 + block.pos.1);
                    
                    // Check bounds
                    if !(0..Grid::WIDTH).contains(&g_x) { return true }

                    // Return collision on lower bound
                    if g_y >= Grid::HEIGHT { return true }

                    // Ignore upper bound (to allow dropping pieces from above the board)
                    if g_y < 0 { continue; }

                    // Check other block
                    if !self[g_y as usize][g_x as usize].is_empty() { return true }
                }
            }
        }

        false
    }

    fn emplace(&mut self, block: &Piece) {

        for y in 0..block.shape.len() {
            for x in 0..block.shape[y].len() {
                let square = block.shape_rotation()[y][x];
                if square != 0 {

                    let (g_x, g_y) = (x as i16 + block.pos.0, y as i16 + block.pos.1);
                    
                    // Check bounds
                    if !(0..Grid::WIDTH).contains(&g_x) { continue; }
                    if !(0..Grid::HEIGHT).contains(&g_y) { continue; }

                    if block.color.is_empty() { panic!("ASSIGNING EMPTY COLOR") }

                    self[g_y as usize][g_x as usize] = block.color;
                }
            }
        }
    }

    fn render(&self, stdout: &mut RawTerminal<Stdout>, next_piece: &Piece) {
        let grid = &self.0;

        // Clear up area above grid
        for y in -4..0 {
            Pos(0, y).goto(stdout).unwrap();

            write!(stdout, "{}", " ".repeat(Grid::WIDTH as usize * 2)).unwrap();
        }

        // Draw grid
        Pos(0, 0).goto(stdout).unwrap();
        
        for row in grid {
            for square in row {
                if square.is_empty() {
                    write!(stdout, "` ").unwrap();
                }
                else {
                    write!(stdout, "{}o{} ", square, Fg(termion::color::Reset)).unwrap();
                }
            }

            write!(stdout, "{}{}", cursor::Down(1), cursor::Left(Grid::WIDTH as u16 * 2)).unwrap();
        }

        // Draw next piece box outline
        Pos(Grid::WIDTH + 1, 0).goto(stdout).unwrap();

        for y in 0..=6 {
            for x in 0..=6 {
                match y {
                    0 | 6 => write!(stdout, "--"),
                    _ => match x {
                        0 | 6 => write!(stdout, "| "),
                        _ => write!(stdout, "  ")
                    }
                }.unwrap()
            }

            write!(stdout, "{}{}", cursor::Down(1), cursor::Left(14)).unwrap();
        }

        // Place label in the center
        Pos(Grid::WIDTH + 3, 0).goto(stdout).unwrap();

        write!(stdout, "Next:").unwrap();

        next_piece.render(stdout);
    }
}

impl Piece {
    fn new() -> Self {
        Piece { 
            shape: Shapes::rand(),
            pos: Pos(Grid::WIDTH + 4, 1),
            color: thread_rng().gen(),
            rotation: 0
        }
    }

    fn move_top(&mut self) {
        self.pos = Pos(Grid::WIDTH / 2, -4);
    }

    fn try_move(&mut self, grid: &Grid, x: i16, y: i16) -> Option<()> {
        self.pos.0 += x;
        self.pos.1 += y;

        if grid.check_collision(self) {
            self.pos.0 -= x;
            self.pos.1 -= y;

            None
        }
        else {
            Some(())
        }
    }

    fn try_rotate(&mut self, grid: &Grid, rotation: u8) -> Option<()> {
        let old = self.rotation;
        self.rotation = rotation;

        if grid.check_collision(self) {
            self.rotation = old;

            None
        }
        else { Some(()) }
    }

    fn clockwise(&self) -> u8 {
        (self.rotation + 1) % 4
    }

    fn anti_clockwise(&self) -> u8 {
        let new = self.rotation as i8 - 1;
        
        if new < 0 { 3 } else { new as u8 }
    }

    fn flipped(&self) -> u8 {
        (self.rotation + 2) % 4
    }

    fn shape_rotation(&self) -> &ShapeRotation {
        &self.shape[self.rotation as usize]
    }

    fn render(&self, stdout: &mut RawTerminal<Stdout>) {
        self.pos.goto(stdout).unwrap();

        for row in self.shape_rotation().0 {
            for square in row {
                if square == 0 {
                    write!(stdout, "{}", cursor::Right(2)).unwrap();
                }
                else {
                    write!(stdout, "{}o{} ", self.color, Fg(termion::color::Reset)).unwrap();
                }
            }

            write!(stdout, "{}{}", cursor::Down(1), cursor::Left(4 * 2)).unwrap();
        }
    }
}

fn main() {
    let mut stdout = stdout().into_raw_mode().unwrap();

    write!(stdout, "{}{}", termion::clear::All, cursor::Hide).unwrap();

    stdout.flush().unwrap();

    let mut grid = Grid( Default::default() );

    let mut piece = Piece::new();
    piece.move_top();

    let mut next_piece = Piece::new();

    let (tx, rx) = mpsc::channel();
    let (stop_tx, stop_rx) = mpsc::channel();

    thread::spawn(move || {
        loop {
            if let Some(c) = stdin().keys().next() {  
                tx.send(c.unwrap()).unwrap();

                thread::sleep(Duration::from_millis(10));
            }

            match stop_rx.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    break;
                },
                _ => ()
            }
        }
    });

    let mut delta_time = Duration::new(0, 0);

    loop {

        let start_time = Instant::now();

        grid.render(&mut stdout, &next_piece);

        piece.render(&mut stdout);

        stdout.flush().unwrap();

        let mut rotated_this_frame = false;

        if let Ok(c) = rx.recv_timeout(Duration::from_millis(100)) {
            match c {
                Key::Char('p') => {
                    stop_tx.send(()).unwrap();
                    break;
                },
                Key::Char('q') => piece.try_move(&grid, -1, 0).unwrap_or(()),
                Key::Char('d') => piece.try_move(&grid, 1, 0).unwrap_or(()),
                Key::Char('z') => piece.try_move(&grid, 0, 1).unwrap_or(()),
                Key::Char('s') => {
                    while piece.try_move(&grid, 0, 1).is_some() {}

                    grid.emplace(&piece);
                    piece = next_piece;
                    piece.move_top();

                    next_piece = Piece::new();
                },
                Key::Left => {
                    if piece.try_rotate(&grid, piece.anti_clockwise()).is_some() {
                        rotated_this_frame = true;
                    }
                },
                Key::Right => {
                    if piece.try_rotate(&grid, piece.clockwise()).is_some() {
                        rotated_this_frame = true;
                    }
                },
                Key::Up => {
                    if piece.try_rotate(&grid, piece.flipped()).is_some() {
                        rotated_this_frame = true;
                    }
                }
                _ => ()
            }            
        }

        while delta_time.as_millis() > 500 {
            if piece.try_move(&grid, 0, 1).is_none() && !rotated_this_frame {
                grid.emplace(&piece);
                piece = next_piece;
                piece.move_top();

                next_piece = Piece::new();
            }

            delta_time -= Duration::from_millis(500);
        }

        delta_time += Instant::now() - start_time;
    }
}
