#![feature(globs)]

extern crate ncurses;
extern crate sync;
extern crate debug;
use ncurses::*;
use sync::{Arc,Mutex};
use std::io::signal::{Listener, Interrupt};

fn move_xy (c: Coordinate) {
    let c = c.zero_based();
    move(c.y as i32, c.x as i32);
}

#[deriving(PartialEq)]
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

#[deriving(Clone,PartialEq)]
struct Coordinate {
    x: uint,
    y: uint
}

impl Coordinate {
    fn up (&mut self)    { self.y-=1; }
    fn down (&mut self)  { self.y+=1; }
    fn left (&mut self)  { self.x-=1; }
    fn right (&mut self) { self.x+=1; }
    fn zero_based(&self) -> Coordinate {
        Coordinate{ x:self.x-1, y:self.y-1 }
    }
}

enum GridError {
    NotFound,
    OutOfBounds,
    Overlapping
}

struct Grid {
    matrix: Vec<Coordinate>,
    cols: uint,
    rows: uint
}

impl Grid {
    fn new(cols: uint, rows: uint) -> Grid {
        Grid {
            matrix: vec!(),
            cols: cols,
            rows: rows
        }
    }

    fn draw(&mut self, coord: Coordinate, sym: &'static str) {
        self.set(coord);
        move_xy(coord);
        printw(sym);
    }

    fn set (&mut self, coord: Coordinate) {
        self.matrix.push(coord);
    }

    fn unset (&mut self, coord: Coordinate) {
        self.matrix.retain(|c| !c.eq(&coord) );
    }

    fn is_empty (&mut self) -> bool {
        self.matrix.len() == 0
    }

    fn center (&mut self) -> Coordinate {
        Coordinate { x:self.cols / 2, y:self.rows / 2 }
    }

    fn clear (&mut self) {
        self.matrix.clear();
    }

    fn has_collitions(&mut self) -> bool {
        let mut temp: Vec<&Coordinate> = vec!();

        for coord in self.matrix.iter() {
            if temp.contains(&coord) {
                return true
            }
            temp.push(coord);
        }

        false
    }
}

struct Stage {
    symbol: &'static str,
    snake: Snake,
}

impl Stage {
    fn new () -> Stage {
        Stage {
            symbol: "X",
            snake: Snake::new(),
            //edibles: vec!()
        }
    }

    fn render (&mut self, grid: &mut Grid) {
        for x in range(1, grid.cols+1) {
            grid.draw(Coordinate{ x:x, y:1 }, self.symbol);
            grid.draw(Coordinate{ x:x, y:grid.rows }, self.symbol);
        }

        for y in range(2, grid.rows) {
            grid.draw(Coordinate{ x:1, y:y }, self.symbol);
            grid.draw(Coordinate{ x:grid.cols, y:y }, self.symbol);
        }

        self.snake.render(grid);
    }

    fn start (&mut self, center: Coordinate) {
        self.snake.start(center);
    }

    fn step(&mut self) {
        self.snake.step();
    }
}


struct Snake {
    position: Coordinate,
    direction: Direction,
    symbol: &'static str,
    moves: Vec<Direction>,
    refreshed: bool
}

impl Snake {
    fn new () -> Snake {
        Snake {
            position: Coordinate{ x:1, y:1 },
            direction: Right,
            symbol: "o",
            refreshed: false,
            moves: vec!(),
        }
    }

    fn start (&mut self, center: Coordinate) {
        self.position = center;

        for _ in range(0, 3) {
            self.moves.push(self.direction);
        }
    }

    fn render (&mut self, grid: &mut Grid) {
        let mut tail = self.position.clone();

        for &m in self.moves.iter().rev() {
            grid.draw(tail, self.symbol);
            tail.move(m.inverse())
        }

        tail.move(self.moves.shift().unwrap().inverse());
        grid.unset(Coordinate{ x:tail.x+1, y:tail.y+1 });
        self.refreshed = true;
    }

    fn step (&mut self) {
        self.move(self.direction);
    }
}

trait Movement {
    fn move(&mut self, Direction) {}
}

impl Movement for Coordinate {
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
        if self.refreshed && !direction.inverse().eq(self.moves.last().unwrap()) {
            self.position.move(direction);
            self.moves.push(direction);
            self.direction = direction;
            self.refreshed = false;
        }
    }
}

struct Game {
    stage: Stage,
    grid: Grid
}

impl Game {
    fn new (cols: uint, rows: uint) -> Game {
        Game {
            stage: Stage::new(),
            grid: Grid::new(cols, rows)
        }
    }

    fn render (&mut self) {
        clear();
        self.grid.clear();
        self.stage.render(&mut self.grid);
        if self.grid.has_collitions() {
            printw("BOOM!");
        }
        refresh();
    }

    fn start (&mut self) {
        self.stage.start(self.grid.center());
        self.render();
    }

    fn step (&mut self) {
        self.stage.step();
        // potentially detect collisions here
    }

    fn move (&mut self, direction: Direction) {
        self.stage.snake.move(direction);
    }
}

fn main () {
    initscr();
    cbreak();
    noecho();
    curs_set(CURSOR_INVISIBLE);
    keypad(stdscr, true);

    let mut game = Game::new(20,20);
    game.start();

    let mutex = Arc::new(Mutex::new(game));
    let mutex_2 = mutex.clone();

    spawn(proc() {
        loop {
            std::io::timer::sleep(300);
            mutex.lock().step();
            mutex.lock().render();
        }
    });

    spawn(proc() {
        loop {
            match getch() {
                KEY_UP      => { mutex_2.lock().move(Up);    },
                KEY_DOWN    => { mutex_2.lock().move(Down);  },
                KEY_LEFT    => { mutex_2.lock().move(Left);  },
                KEY_RIGHT   => { mutex_2.lock().move(Right); },
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

#[test]
fn test_grid_empty () {
    let mut grid = Grid::new(2, 2);
    assert_eq!(grid.is_empty(), true);
}

#[test]
fn test_grid_set () {
    let mut grid = Grid::new(2, 2);
    grid.set(Coordinate{ x:1, y:2 });
    assert_eq!(grid.is_empty(), false);
}

#[test]
fn test_grid_collition () {
    let mut grid = Grid::new(2, 2);
    grid.set(Coordinate{ x:1, y:1 });
    assert!(grid.has_collitions() == false);
    grid.set(Coordinate{ x:2, y:1 });
    assert!(grid.has_collitions() == false);
    grid.set(Coordinate{ x:2, y:1 });
    assert!(grid.has_collitions() == true);
}
