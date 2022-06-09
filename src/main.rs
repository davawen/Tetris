use std::{io::{stdout, Write, stdin, Stdout}, thread, time::{Duration, Instant}, sync::mpsc::{self, TryRecvError}};
use derive_more::{Deref, DerefMut};

use rand::{thread_rng, Rng};
use termion::{
    cursor,
    raw::{IntoRawMode, RawTerminal},
    input::TermRead,
    event::Key, color::Fg
};

mod shape;
mod color;

use shape::*;
use color::Color;

#[derive(Default, Debug, Deref, DerefMut)]
struct Grid([[Color; 8]; 16]);

#[derive(Debug)]
struct Piece {
    color: Color,
    shape: &'static Shape,
    pos: Pos,
    rotation: u8
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

    fn remove_full_lines(&mut self) {
        let grid = &mut self.0;
            
        for y in 0..grid.len() {
            let row = &mut grid[y];

            if !row.iter().any(|c|{ c.is_empty() }) {
                row.fill(Color::Empty);

                grid[0..=y].rotate_right(1);
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
                    write!(stdout, "{}▣{} ", square, Fg(termion::color::Reset)).unwrap();
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
                    write!(stdout, "{}▣{} ", self.color, Fg(termion::color::Reset)).unwrap();
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
                    grid.remove_full_lines();

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
                grid.remove_full_lines();

                piece = next_piece;
                piece.move_top();
                next_piece = Piece::new();
            }

            delta_time -= Duration::from_millis(500);
        }

        delta_time += Instant::now() - start_time;
    }
}
