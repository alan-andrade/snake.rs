#![feature(globs)]

extern crate ncurses;
extern crate sync;
use ncurses::*;
use sync::{Arc,Mutex};
use std::io::signal::{Listener, Interrupt};

type State = uint;

static WALL: &'static str = "#";
static ON:  uint = 1;
static OFF: uint = 0;

fn move_xy (x: i32, y: i32) {
    move(y-1,x-1);
}

enum Direction {
    Up,
    Down,
    Left,
    Right
}

impl Direction {
    fn inverse (&self) -> Direction {
        match *self {
            Up => Down,
            Down => Up,
            Left => Right,
            Right => Left
        }
    }
}

struct Position {
    x: i32,
    y: i32
}

impl Position {
    fn up (&mut self)    { self.y-=1; }
    fn down (&mut self)  { self.y+=1; }
    fn left (&mut self)  { self.x-=1; }
    fn right (&mut self) { self.x+=1; }
}

struct Coordinate {
    x: uint,
    y: uint
}

struct Grid {
    set: Vec<uint>,
    cols: uint,
    rows: uint
}

enum GridError {
    NotFound,
    OutOfBounds
}

impl Grid {
    fn new(cols: uint, rows: uint) -> Grid {
        let mut set = vec!();

        for _ in range(0,rows*cols) {
            set.push(0);
        }

        Grid {
            set: set,
            cols: cols,
            rows: rows
        }
    }

    fn index_for(&mut self, x: uint, y: uint) -> Result<uint, GridError> {
        if x > self.cols || y > self.rows {
            return Err(OutOfBounds)
        }

        Ok((x-1) + (self.rows * (y-1)))
    }

    fn turn(&mut self, state: State, x: uint, y: uint) -> Result<Coordinate, GridError> {
        let target_index = try!(self.index_for(x, y));

        match state {
            ON  => *self.set.get_mut(target_index) = ON,
            OFF => *self.set.get_mut(target_index) = OFF,
            _ => {}
        }

        Ok(Coordinate{x: x, y: y})
    }

    fn is_empty (&mut self) -> bool {
        self.set.iter().all(|&x| x == OFF)
    }
}

struct GridSet<'a> {
    grids: Vec<&'a Grid>
}

impl<'a>  GridSet<'a> {
    fn new<'a> (grids: Vec<&'a Grid>) -> GridSet<'a> {
        GridSet {
            grids: grids
        }
    }

    fn resolve (&'a self) {
        //let x = self.grids.get(0).width.clone();
        //let y = self.grids.get(0).height.clone();
        //let max = self.grids.len();

        //for counter in range(0, x * y) {
            //let (x_count, y_count) = (0,0);

            //for &&grid in self.grids.iter() {
                //spot_counter = spot_counter + *grid.set.get(counter as uint)
            //}
        //}
    }
}

struct Stage {
    cols: i32,
    rows: i32
}

impl Stage {
    fn new (cols: i32, rows: i32) -> Stage {
        Stage {
            cols: cols-1,
            rows: rows-1
        }
    }

    fn render (&self, grid: &mut Box<Grid>) {
        for x in range(1, self.cols) {
            move_xy(x, 1);
            grid.turn(ON, x as uint, 1);
            printw(WALL);

            move_xy(x, self.rows);
            grid.turn(ON, x as uint, self.rows as uint);
            printw(WALL);
        }

        for y in range(2, self.rows) {
            move_xy(1, y);
            printw(WALL);
            grid.turn(ON, 1, y as uint);
            move_xy(self.cols, y);
            printw(WALL);
            grid.turn(ON, self.cols as uint, y as uint);
        }
    }

    fn center (&self) -> (i32,i32) {
        (self.cols / 2, self.rows / 2)
    }
}


struct Snake {
    position: Position,
    direction: Direction,
    symbol: &'static str,
    moves: Vec<Direction>,
    refreshed: bool
}

impl Snake {
    fn new () -> Snake {
        Snake {
            position: Position {x:0, y:0},
            direction: Right,
            symbol: "o",
            refreshed: false,
            moves: vec!(),
        }
    }

    fn init (&mut self, stage: &Stage) {
        let (x,y) = stage.center();
        self.position.x = x;
        self.position.y = y;
        for _ in range(0, 3) {
            self.moves.push(self.direction);
        }
    }

    fn render (&mut self, grid: &mut Box<Grid>) {
        let mut tail = Position {
            x: self.position.x,
            y: self.position.y
        };

        for &m in self.moves.iter().rev() {
            move(tail.y, tail.x);
            printw(self.symbol);
            tail.move(m.inverse())
        }

        self.moves.shift();
        self.refreshed = true;
    }

    fn step (&mut self) {
        self.move(self.direction);
    }
}

trait Movement {
    fn move(&mut self, Direction) {}
}


impl Movement for Position {
    fn move (&mut self, direction: Direction) {
        match direction {
            Up      => self.up(),
            Down    => self.down(),
            Left    => self.left(),
            Right   => self.right()
        }
    }
}

impl Movement for Snake {
    fn move(&mut self, direction: Direction) {
        if self.refreshed {
            self.position.move(direction);
            self.moves.push(direction);
            self.direction = direction;
            self.refreshed = false;
        }
    }
}

struct Game {
    snake: Snake,
    stage: Stage,
    grid: Box<Grid>
}

impl Game {
    fn new (cols: i32, rows: i32) -> Game {
        Game {
            stage: Stage::new(cols, rows),
            snake: Snake::new(),
            grid: box Grid::new(cols as uint, rows as uint)
        }
    }

    fn render (&mut self) {
        clear();
        self.stage.render(&mut self.grid);
        self.snake.render(&mut self.grid);
        refresh();
    }

    fn start (&mut self) {
        self.snake.init(&self.stage);
        self.render();
    }
}

fn main () {
    initscr();
    cbreak();
    noecho();
    curs_set(CURSOR_INVISIBLE);
    keypad(stdscr, true);

    let mut game = Game::new(30,30);

    game.start();

    let mutex = Arc::new(Mutex::new(game));
    let mutex_2 = mutex.clone();

    spawn(proc() {
        loop {
            std::io::timer::sleep(500);
            mutex.lock().snake.step();
            mutex.lock().render();
        }
    });

    spawn(proc() {
        loop {
            match getch() {
                KEY_UP      => { mutex_2.lock().snake.move(Up);    },
                KEY_DOWN    => { mutex_2.lock().snake.move(Down);  },
                KEY_LEFT    => { mutex_2.lock().snake.move(Left);  },
                KEY_RIGHT   => { mutex_2.lock().snake.move(Right); },
                _ => { }
            }
        }
    });

    let mut listener = Listener::new();
    listener.register(Interrupt);

    loop {
        match listener.rx.recv() {
            Interrupt => { endwin(); },
            _ => (),
        }
    }
}

#[cfg(test)]
#[test]
fn test_grid_empty () {
    let mut grid = Grid::new(2, 2);
    assert_eq!(grid.is_empty(), true);
}

#[test]
fn test_grid_turn () {
    let mut grid = Grid::new(2, 2);
    grid.turn(ON, 1, 2);
    assert_eq!(grid.is_empty(), false);
}

#[test]
fn test_grid_out_of_bounds () {
    let mut grid = Grid::new(3, 3);
    match grid.turn(OFF, 3, 4) {
        Err(e) => match e {
            OutOfBounds => assert!(true),
            _ => {}
        },
        Ok(_) => {}
    }
}

#[test]
fn test_grid_index_of () {
    let mut grid = Grid::new(3,3);
    match grid.index_for(3,3) {
        Ok(c) => { assert_eq!(c, 8) }
        Err(_) => {}
    }

    match grid.index_for(1,1) {
        Ok(c) => { assert_eq!(c, 0) }
        Err(_) => {}
    }

    match grid.index_for(3,2) {
        Ok(c) => { assert_eq!(c, 5) }
        Err(_) => {}
    }

    match grid.index_for(2,2) {
        Ok(c) => { assert_eq!(c, 4) }
        Err(_) => {}
    }

    match grid.index_for(1,3) {
        Ok(c) => { assert_eq!(c, 6) }
        Err(_) => {}
    }
}
