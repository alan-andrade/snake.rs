#![feature(globs)]

extern crate ncurses;
extern crate sync;
extern crate debug;
use ncurses::*;
use sync::{Arc,Mutex};
use std::io::signal::{Listener, Interrupt};

fn move_to (c: Coordinate) {
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
    Collision(Coordinate)
}

impl GridError {
    fn to_str(&self) -> String {
        match *self {
            Collision(coord) => {
                let mut msg = String::new();
                msg.push_str(coord.x.to_str().as_slice());
                msg.push_str(" ,");
                msg.push_str(coord.y.to_str().as_slice());
                msg
            }
        }
    }
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

    fn draw(&mut self, coord: Coordinate, sym: &'static str) -> Result<(), GridError> {
        match self.set(coord) {
            Ok(()) => {
                move_to(coord);
                printw(sym);
                Ok(())
            },
            Err(e) => { Err(e) }
        }
    }

    fn set (&mut self, coord: Coordinate) -> Result<(), GridError> {
        if self.matrix.contains(&coord) {
            return Err(Collision(coord))
        }
        Ok(self.matrix.push(coord))
    }

    fn unset (&mut self, coord: Coordinate) {
        self.matrix.retain(|c| !c.eq(&coord) );
    }

    fn center (&mut self) -> Coordinate {
        Coordinate { x:self.cols / 2, y:self.rows / 2 }
    }

    fn clear (&mut self) {
        self.matrix.clear();
    }

    fn random_free_spot(&mut self) -> Coordinate {
        use std::rand::{task_rng, sample};

        let mut free_spots = vec!();
        for x in range(1, self.cols+1) {
            for y in range(x, self.rows+1) {
                let coord = Coordinate{ x:x, y:y };
                if !self.matrix.contains(&coord) {
                    free_spots.push(coord)
                }
            }
        }

        let mut rng = task_rng();
        let index = sample(&mut rng, range(0, free_spots.len()), 1);
        free_spots.remove(*index.get(0)).unwrap()
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
            snake: Snake::new()
        }
    }

    #[allow(unused_must_use)]
    fn render (&mut self, grid: &mut Grid) -> Result<(), GridError> {
        for x in range(1, grid.cols+1) {
            grid.draw(Coordinate{ x:x, y:1 }, self.symbol);
            grid.draw(Coordinate{ x:x, y:grid.rows }, self.symbol);
        }

        for y in range(2, grid.rows) {
            grid.draw(Coordinate{ x:1, y:y }, self.symbol);
            grid.draw(Coordinate{ x:grid.cols, y:y }, self.symbol);
        }

        self.snake.render(grid)
    }

    fn start (&mut self, center: Coordinate) {
        self.snake.start(center);
    }

    fn step(&mut self) {
        self.snake.step();
    }
}

struct Apple {
    position: Coordinate,
    has_a_place: bool,
    symbol: &'static str,
}

impl Apple {
    fn new () -> Apple {
        Apple {
            position: Coordinate{ x:1, y:1 },
            has_a_place: false,
            symbol: "A"
        }
    }
}


struct Snake {
    position: Coordinate,
    direction: Direction,
    symbol: &'static str,
    moves: Vec<Direction>,
    refreshed: bool,
    apple: Apple
}

impl Snake {
    fn new () -> Snake {
        Snake {
            position: Coordinate{ x:1, y:1 },
            direction: Right,
            symbol: "o",
            refreshed: false,
            moves: vec!(),
            apple: Apple::new()
        }
    }

    fn start (&mut self, center: Coordinate) {
        self.position = center;

        for _ in range(0, 3) {
            self.moves.push(self.direction);
        }
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

impl Movement for Game {
    fn move(&mut self, direction: Direction) {
        self.stage.move(direction)
    }
}

impl Movement for Stage {
    fn move(&mut self, direction: Direction) {
        self.snake.move(direction)
    }
}

trait Render {
    fn render (&mut self, grid: &mut Grid) -> Result<(), GridError>;
    fn step (&mut self);
}

impl Render for Snake {
    fn render (&mut self, grid: &mut Grid) -> Result<(), GridError> {
        let mut tail = self.position.clone();

        for &m in self.moves.iter().rev() {
            match grid.draw(tail, self.symbol) {
                Err(e) => return Err(e),
                Ok(_)  => tail.move(m.inverse())
            }
        }

        self.refreshed = true;

        match self.apple.render(grid) {
            Err(_) => { Ok(()) }
            Ok(_) => {
                tail.move(self.moves.shift().unwrap().inverse());
                grid.unset(Coordinate{ x:tail.x+1, y:tail.y+1 });
                Ok(())
            }
        }
    }

    fn step (&mut self) {
        self.move(self.direction);
    }
}

impl Render for Apple {
    fn render (&mut self, grid: &mut Grid) -> Result<(), GridError> {
        if !self.has_a_place {
            self.position = grid.random_free_spot();
            self.has_a_place = true
        }

        match grid.draw(self.position, self.symbol) {
            Err(e) => {
                self.has_a_place = false;
                Err(e)
            },
            _ => Ok(())
        }
    }

    fn step (&mut self) {
        //
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

    #[allow(unused_must_use)]
    fn start (&mut self) {
        self.stage.start(self.grid.center());
        self.render();
    }

    fn render (&mut self) -> Result<(), GridError> {
        clear();
        self.grid.clear();
        let result = match self.stage.render(&mut self.grid) {
            Err(e) => {
                let mut msg = String::from_str("Collision on ");
                msg.push_str(e.to_str().as_slice());
                printw(msg.as_slice());

                move_to(self.grid.center());
                printw("Game Over");
                Err(e)
            }
            Ok(n) => { Ok(n) }
        };

        refresh();

        result
    }

    fn step (&mut self) {
        self.stage.step();
    }
}

fn main () {
    initscr();
    cbreak();
    noecho();
    curs_set(CURSOR_INVISIBLE);
    keypad(stdscr, true);

    let mut game = Game::new(20,15);
    game.start();

    let mutex = Arc::new(Mutex::new(game));
    let mutex_2 = mutex.clone();

    spawn(proc() {
        loop {
            std::io::timer::sleep(300);
            mutex.lock().step();
            match mutex.lock().render() {
                Err(_) => { break; },
                Ok(_) => {}
            }
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
fn test_grid_collition () {
    let mut grid = Grid::new(2, 2);
    grid.set(Coordinate{ x:1, y:1 });
    assert!(grid.has_collitions() == false);
    grid.set(Coordinate{ x:2, y:1 });
    assert!(grid.has_collitions() == false);
    grid.set(Coordinate{ x:2, y:1 });
    assert!(grid.has_collitions() == true);
}
