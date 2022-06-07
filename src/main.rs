use std::{io::{stdout, Write, stdin, Stdout}, thread, time::Duration, collections::{vec_deque, VecDeque}, sync::{Mutex, mpsc::{self, TryRecvError}}};
use derive_more::{Deref, DerefMut};

use termion::{
    cursor,
    raw::{IntoRawMode, RawTerminal},
    input::{Events, TermRead},
    event::{Key, Event}
};

#[derive(Debug)]
enum Color {
    Empty,
    White
}

#[derive(Debug, Clone, Copy, Default)]
struct Pos(i16, i16);

impl Pos {
    fn to_term(&self) -> Pos {
        Pos( self.0 * 2 + 1, self.1 + 1 )
    }
}

#[derive(Default, Debug, Deref, DerefMut)]
struct ShapeRotation([[u16; 4]; 4]);

type Shape = [ShapeRotation; 4];

#[derive(Default, Debug, Deref, DerefMut)]
struct Grid([[u16; 8]; 16]);

#[derive(Debug)]
struct Block {
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
            [ 1, 0, 0, 0 ],
            [ 1, 0, 0, 0 ],
            [ 1, 0, 0, 0 ],
            [ 1, 0, 0, 0 ]
        ]),
        ShapeRotation([
            [ 0, 0, 0, 0 ],
            [ 0, 0, 0, 0 ],
            [ 0, 0, 0, 0 ],
            [ 1, 1, 1, 1 ]
        ]),
        ShapeRotation([
            [ 1, 0, 0, 0 ],
            [ 1, 0, 0, 0 ],
            [ 1, 0, 0, 0 ],
            [ 1, 0, 0, 0 ]
        ]),
        ShapeRotation([
            [ 0, 0, 0, 0 ],
            [ 0, 0, 0, 0 ],
            [ 0, 0, 0, 0 ],
            [ 1, 1, 1, 1 ]
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
}

impl Grid {
    const WIDTH: i16 = 8;
    const HEIGHT: i16 = 16;

    fn check_collision(&self, block: &Block) -> bool {
        
        for y in 0..block.shape.len() {
            for x in 0..block.shape[y].len() {
                // Only check for actual blocks in the block
                if block.shape_rotation()[y][x] != 0 {

                    let (g_x, g_y) = (x as i16 + block.pos.0, y as i16 + block.pos.1);
                    
                    // Check bounds
                    if !(0..Grid::WIDTH).contains(&g_x) { return true }
                    if !(0..Grid::HEIGHT).contains(&g_y) { return true }

                    // Check other block
                    if self[g_y as usize][g_x as usize] != 0 { return true }
                }
            }
        }

        false
    }

    fn emplace(&mut self, block: &Block) {

        for y in 0..block.shape.len() {
            for x in 0..block.shape[y].len() {
                let square = block.shape_rotation()[y][x];
                if square != 0 {

                    let (g_x, g_y) = (x as i16 + block.pos.0, y as i16 + block.pos.1);
                    
                    // Check bounds
                    if !(0..Grid::WIDTH).contains(&g_x) { panic!() }
                    if !(0..Grid::HEIGHT).contains(&g_y) { panic!() }

                    // Check other block
                    self[g_y as usize][g_x as usize] = square;
                }
            }
        }
    }

    fn render(&self, stdout: &mut RawTerminal<Stdout>) {
        let grid = &self.0;

        write!(stdout, "{}", cursor::Goto(1, 1)).unwrap();

        for row in grid {
            for &square in row {
                if square == 0 {
                    write!(stdout, "` ").unwrap();
                }
                else {
                    write!(stdout, "o ").unwrap();
                }
            }

            write!(stdout, "{}{}", cursor::Down(1), cursor::Left(1000)).unwrap();
        }
    }
}

impl Block {
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

    fn shape_rotation(&self) -> &ShapeRotation {
        &self.shape[self.rotation as usize]
    }

    fn render(&self, stdout: &mut RawTerminal<Stdout>) {
        let pos = self.pos.to_term();
        write!(stdout, "{}", cursor::Goto(pos.0.try_into().unwrap(), pos.1.try_into().unwrap())).unwrap();

        for row in self.shape_rotation().0 {
            for square in row {
                if square == 0 {
                    write!(stdout, "{}", cursor::Right(2)).unwrap();
                }
                else {
                    write!(stdout, "o ").unwrap();
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

    let mut piece = Block { shape: &Shapes::I, pos: Pos(1, 1), color: Color::White, rotation: 0 };

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

    let mut ticks = 0;

    loop {

        if let Ok(c) = rx.try_recv() {
            match c {
                Key::Char('q') => {
                    stop_tx.send(()).unwrap();
                    break;
                },
                Key::Left => piece.try_move(&grid, -1, 0).unwrap_or(()),
                Key::Right => piece.try_move(&grid, 1, 0).unwrap_or(()),
                Key::Up => piece.try_rotate(&grid, piece.clockwise()).unwrap_or(()),
                _ => ()
            }            
        }

        if ticks % 500 == 0 {
            if piece.try_move(&grid, 0, 1).is_none() {
                grid.emplace(&piece);
                piece = Block { shape: &Shapes::I, pos: Pos(1, 1), color: Color::White, rotation: 0 };
            }
        }

        grid.render(&mut stdout);

        piece.render(&mut stdout);

        stdout.flush().unwrap();

        thread::sleep(Duration::from_millis(50));

        ticks += 50;
    }
}
