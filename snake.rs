#![feature(globs)]

extern crate ncurses;
extern crate sync;
use ncurses::*;
use sync::{Arc,Mutex};
use std::io::signal::{Listener, Interrupt};

struct Position {
    x: i32,
    y: i32
}

impl Position {
    fn up (&mut self)    { self.y-=1; }
    fn down (&mut self)  { self.y+=1; }
    fn left (&mut self)  { self.x-=1; }
    fn right (&mut self) { self.x+=1; }
    fn move (&mut self, direction: Direction) {
        match direction {
            Up => self.up(),
            Down => self.down(),
            Left => self.left(),
            Right => self.right()
        }
    }
}

static WALL: &'static str = "#";

struct Stage {
    cols: i32,
    rows: i32
}

impl Stage {
    fn new (cols: i32, rows: i32) -> Stage {
        Stage { cols: cols, rows: rows }
    }

    fn draw (&self) {
        for x in range(0, self.cols) {
            move(0, x);
            printw(WALL);
            move(self.rows-1, x);
            printw(WALL);
        }

        for y in range(1, self.rows) {
            move(y, 0);
            printw(WALL);
            move(y, self.cols-1);
            printw(WALL);
        }
    }

    fn center (&self) -> (i32,i32) {
        (self.cols / 2, self.rows / 2)
    }
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

    fn draw (&mut self) {
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

    fn move (&mut self, direction: Direction) {
        if self.refreshed {
            self.position.move(direction);
            self.moves.push(direction);
            self.direction = direction;
            self.refreshed = false;
        }
    }

    fn step (&mut self) {
        self.move(self.direction);
    }
}

struct Game {
    snake: Snake,
    stage: Stage
}

impl Game {
    fn draw (&mut self) {
        clear();
        self.stage.draw();
        self.snake.draw();
        refresh();
    }

    fn start (&mut self) {
        self.snake.init(&self.stage);
        self.draw();
    }
}

fn main () {
    initscr();
    cbreak();
    noecho();
    curs_set(CURSOR_INVISIBLE);
    keypad(stdscr, true);

    let mut game = Game {
        stage: Stage::new(40, 40),
        snake: Snake::new()
    };

    game.start();

    let mutex = Arc::new(Mutex::new(game));
    let mutex_2 = mutex.clone();

    spawn(proc() {
        loop {
            std::io::timer::sleep(500);
            mutex.lock().snake.step();
            mutex.lock().draw();
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
