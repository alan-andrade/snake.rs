#![feature(globs)]

extern crate ncurses;
extern crate sync;
extern crate debug;
use ncurses::*;
use sync::{Arc,Mutex};
use std::io::signal::{Listener, Interrupt};
use std::comm::channel;

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
    fn zero_based(&self) -> Coordinate {
        Coordinate{ x:self.x-1, y:self.y-1 }
    }
}

enum GridResult {
    Grow,
    Step,
    Collision(Coordinate)
}

trait Drawable {
    pub fn draw(&self, &'static str, Coordinate);
}

struct NDrawer;

impl Drawable for NDrawer {
    fn draw(string: &'static str, coordinate: Coordinate) {
        NDrawer::move_to(coordinate);
        printw(string);
    }

}

impl NDrawer {
    fn move_to (c: Coordinate) {
        let c = c.zero_based();
        move(c.y as i32, c.x as i32);
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

    fn draw(&mut self, coord: Coordinate, sym: &'static str) -> GridResult {
        match self.set(coord) {
            Step | Grow => {
                NDrawer::draw(sym, coord);
                Step
            },
            Collision(c) => { Collision(c) }
        }
    }

    fn set (&mut self, coord: Coordinate) -> GridResult {
        if self.matrix.contains(&coord) {
            return Collision(coord)
        }
        self.matrix.push(coord);
        Step
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
            symbol: "+",
            snake: Snake::new()
        }
    }

    #[allow(unused_must_use)]
    fn render (&mut self) -> GridResult {
        for x in range(1, grid.cols+1) {
            Drawer::draw(Coordinate{ x:x, y:1 }, self.symbol);
            Drawer::draw(Coordinate{ x:x, y:grid.rows }, self.symbol);
        }

        for y in range(2, grid.rows) {
            Drawer::draw(Coordinate{ x:1, y:y }, self.symbol);
            Drawer::draw(Coordinate{ x:grid.cols, y:y }, self.symbol);
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
            symbol: "O",
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
            Up      => self.y-=1,
            Down    => self.y+=1,
            Left    => self.x-=1,
            Right   => self.x+=1
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

// XXX: A game doesnt move
impl Movement for Game {
    fn move(&mut self, direction: Direction) {
        self.stage.move(direction)
    }
}

// XXX: A stage doesnt move
impl Movement for Stage {
    fn move(&mut self, direction: Direction) {
        self.snake.move(direction)
    }
}

trait Render {
    fn render (&mut self, grid: &mut Grid) -> GridResult;
    fn step (&mut self);
}

impl Render for Snake {
    fn render (&mut self, grid: &mut Grid) -> GridResult {
        let mut tail = self.position.clone();

        for &m in self.moves.iter().rev() {
            Drawer::draw(tail, self.symbol);
            tail.move(m.inverse());
        }

        self.refreshed = true;

        match self.apple.render(grid) {
            Step => {
                tail.move(self.moves.shift().unwrap().inverse());
                grid.unset(Coordinate{ x:tail.x+1, y:tail.y+1 });
                Step
            },
            _var => { _var }
        }
    }

    fn step (&mut self) {
        self.move(self.direction);
    }
}

impl Render for Apple {
    fn render (&mut self, grid: &mut Grid) -> GridResult {
        if !self.has_a_place {
            self.position = grid.random_free_spot();
            self.has_a_place = true
        }

        match grid.draw(self.position, self.symbol) {
            Collision(_) => {
                self.has_a_place = false;
                Grow
            },
            _t => { _t }
        }
    }

    fn step (&mut self) { }
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

    fn render (&mut self) -> GridResult {
        clear();
        self.grid.clear();

        let result = match self.stage.render(&mut self.grid) {
            Collision(c) => {
                //let mut msg = String::from_str("Collision on ");
                //msg.push_str(c.x.to_str().as_slice());
                //msg.push_str(" ,");
                //msg.push_str(c.y.to_str().as_slice());
                //printw(msg.as_slice());

                //move_to(self.grid.center());
                //printw("Game Over");
                Collision(c)
            }
            _type => { _type }
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
    halfdelay(1);
    noecho();
    curs_set(CURSOR_INVISIBLE);
    keypad(stdscr, true);

    let mut game = Game::new(20,20);
    game.start();


    let mutex = Arc::new(Mutex::new(game));
    let mutex_2 = mutex.clone();

    let (tx, rx) =  channel();

    spawn(proc() {
        let mut timer = 300;
        let mut score = 0;

        loop {
            std::io::timer::sleep(timer);
            mutex.lock().step();
            match mutex.lock().render() {
                Collision(_) => { break; },
                Grow => {
                    timer -= 25;
                    score += 1;
                },
                _ => {}
            }
        }

        tx.send(0);
    });

    spawn(proc() {
        loop {
            match getch() {
                KEY_UP      => { mutex_2.lock().move(Up);    },
                KEY_DOWN    => { mutex_2.lock().move(Down);  },
                KEY_LEFT    => { mutex_2.lock().move(Left);  },
                KEY_RIGHT   => { mutex_2.lock().move(Right); },
                ERR => {
                    match rx.try_recv() {
                        Ok(_) => { break; }
                        Err(_) => { }
                    }
                },
                _ => { }
            }
        }

        endwin();
    });

    let mut listener = Listener::new();
    listener.register(Interrupt);

    loop {
        match listener.rx.recv() {
            Interrupt => { endwin(); },
            _ => { break; }
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
