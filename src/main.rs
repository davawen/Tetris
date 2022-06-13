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

#[derive(Debug, Clone)]
struct Piece {
    color: Color,
    shape: Shape,
    pos: Pos,
    rotation: usize
}

fn draw_box(stdout: &mut RawTerminal<Stdout>, pos: Pos, size: Size) {
    pos.goto(stdout).unwrap();

    // ┌─┐
    // │ │
    // └─┘

    let size = size.term_value();

    if size.0 < 2 || size.1 < 2 { panic!("Cannot draw 1x1 box") }

    write!(stdout, "┌{}┐", "─".repeat((size.0 - 2) as usize)).unwrap();

    for _ in 1..size.1-1 {
        write!(stdout, "{}{}", cursor::Left(size.0 as u16), cursor::Down(1)).unwrap();
        write!(stdout, "│{}│", " ".repeat((size.0 - 2) as usize)).unwrap();
    }

    write!(stdout, "{}{}", cursor::Left(size.0 as u16), cursor::Down(1)).unwrap();
    write!(stdout, "└{}┘", "─".repeat((size.0 - 2) as usize)).unwrap();
}

impl Piece {
    fn new() -> Self {
        Piece { 
            shape: Shapes::rand(),
            pos: Pos(Grid::WIDTH + 2, 1),
            color: thread_rng().gen(),
            rotation: 0
        }
    }

    fn hold_piece_pos() -> Pos {
        Pos(Grid::WIDTH + 2, 7)
    }

    fn move_top(&mut self) {
        self.pos = Pos(Grid::WIDTH / 2, -4);
    }

    fn move_by(&mut self, value: Pos) {
        self.pos += value;
    }

    /// Returns true if the move was legal, and false if it wasn't
    fn try_move(&mut self, grid: &Grid, value: Pos) -> bool {
        self.move_by(value);

        if grid.check_collision(self) {
            self.move_by(-value);

            false
        }
        else {
            true
        }
    }

    fn drop(&mut self, grid: &Grid) {
        while self.try_move(grid, Pos(0, 1)) {}
    }

    /// Returns true if the rotation was legal and false if it wasn't
    fn try_rotate(&mut self, grid: &Grid, rotation: usize) -> bool {
        let old = self.rotation;
        self.rotation = rotation;

        if grid.check_collision(self) {
            self.rotation = old;

            false
        }
        else {
            true
        }
    }

    /// Try to rotate piece, and jerks it left and right in case the original rotation isn't possible, to allow for wall bounces
    /// Returns true if a rotation happened and false if none was correct
    fn try_rotate_with_bounce(&mut self, grid: &Grid, rotation: usize) -> bool {
        if self.try_rotate(grid, rotation) { return true }

        let mut try_move_rotation = |v| {
            self.move_by(v);
            if self.try_rotate(grid, rotation) { true }
            else {
                self.move_by(-v);
                false
            }
        };

        if try_move_rotation(Pos(1, 0)) { return true; }
        if try_move_rotation(Pos(-1, 0)) { return true; }

        false
    }

    fn clockwise(&self) -> usize {
        (self.rotation + 1) % self.shape.len()
    }

    fn anti_clockwise(&self) -> usize {
        let new = self.rotation as isize - 1;
        
        if new < 0 { self.shape.len() - 1 } else { new as usize }
    }

    fn flipped(&self) -> usize {
        (self.rotation + 2) % self.shape.len()
    }

    fn shape_rotation(&self) -> &ShapeRotation {
        if self.rotation >= self.shape.len() { panic!("aaa") }

        &self.shape[self.rotation]
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

impl Grid {
    const WIDTH: i16 = 8;
    const HEIGHT: i16 = 16;

    fn check_collision(&self, block: &Piece) -> bool {
        let shape = block.shape_rotation();

        for y in 0..shape.len() {
            for x in 0..shape[y].len() {
                // Only check for actual blocks in the block
                if shape[y][x] != 0 {

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
        let shape = block.shape_rotation();

        for y in 0..shape.len() {
            for x in 0..shape[y].len() {
                let square = shape[y][x];
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

    fn render(&self, stdout: &mut RawTerminal<Stdout>, next_piece: &Piece, hold_piece: &Option<Piece>) {
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

        // "Next Piece" box
        draw_box(stdout, Pos(Grid::WIDTH + 1, 0), Size(6, 6));

        Pos(Grid::WIDTH + 2, 0).goto(stdout).unwrap();
        write!(stdout, "Next:").unwrap();

        next_piece.render(stdout);

        // "Holded Piece" box
        draw_box(stdout, Pos(Grid::WIDTH + 1, 6), Size(6, 6));

        Pos(Grid::WIDTH + 2, 6).goto(stdout).unwrap();
        write!(stdout, "On Hold:").unwrap();

        if let Some(hold_piece) = hold_piece {
            hold_piece.render(stdout);
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
    let mut hold_piece: Option<Piece> = None;

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
    let mut last_move = Instant::now();

    loop {

        let start_time = Instant::now();

        grid.render(&mut stdout, &next_piece, &hold_piece);

        // Render shadow of where the piece would be if it was dropped
        let mut dropped_piece = Piece { color: Color::Gray, ..piece.clone() };
        dropped_piece.drop(&grid);
        
        dropped_piece.render(&mut stdout);
        piece.render(&mut stdout);

        stdout.flush().unwrap();

        if let Ok(c) = rx.recv_timeout(Duration::from_millis(100)) {
            match c {
                Key::Char('p') => {
                    stop_tx.send(()).unwrap();
                    break;
                },
                Key::Char('q') => {
                    piece.try_move(&grid, Pos(-1, 0));
                    last_move = Instant::now();
                },
                Key::Char('d') => {
                    piece.try_move(&grid, Pos(1, 0));
                    last_move = Instant::now();
                },
                Key::Char('z') => { 
                    if piece.try_move(&grid, Pos(0, 1)) {
                        last_move = Instant::now(); // Allow for movement after fast drop
                    }
                },
                Key::Char('s') => {
                    piece.drop(&grid);

                    grid.emplace(&piece);
                    grid.remove_full_lines();

                    piece = next_piece;
                    piece.move_top();
                    next_piece = Piece::new();

                    last_move = Instant::now();
                },
                Key::Left => {
                    if piece.try_rotate_with_bounce(&grid, piece.anti_clockwise()) {
                        last_move = Instant::now();
                    }
                },
                Key::Right => {
                    if piece.try_rotate_with_bounce(&grid, piece.clockwise()) {
                        last_move = Instant::now();
                    }
                },
                Key::Up => {
                    if piece.try_rotate_with_bounce(&grid, piece.flipped()) {
                        last_move = Instant::now();
                    }
                },
                // Any uppercase character means hold
                Key::Char(' ') => {
                    piece.pos = Piece::hold_piece_pos();

                    if let Some(holded) = &mut hold_piece {
                        std::mem::swap(holded, &mut piece);
                    } else {
                        hold_piece = Some(piece);
                        piece = next_piece;
                        next_piece = Piece::new();
                    }

                    piece.move_top();
                }
                _ => ()
            };
        }

        while delta_time.as_millis() > 500 {
            if !piece.try_move(&grid, Pos(0, 1)) && last_move.elapsed().as_millis() > 500 {
                grid.emplace(&piece);
                grid.remove_full_lines();

                piece = next_piece;
                piece.move_top();
                next_piece = Piece::new();
            }

            delta_time -= Duration::from_millis(500);
        }

        delta_time += start_time.elapsed();
    }

    Pos(0, Grid::HEIGHT).goto(&mut stdout).unwrap();
    write!(stdout, "{}", cursor::Show).unwrap();
}
