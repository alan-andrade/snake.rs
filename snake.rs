#![feature(globs)]

extern crate ncurses;
use ncurses::*;

static KEY_Q: i32 = 'q' as i32;

struct Position {
    x: i32,
    y: i32
}

impl Position {
    fn up(&mut self)    { self.y = self.y-1; }
    fn down(&mut self)  { self.y = self.y+1; }
    fn left(&mut self)  { self.x = self.x-1; }
    fn right(&mut self) { self.x = self.x+1; }
}

static WALL: &'static str = "#";

struct Stage {
    cols: i32,
    rows: i32
}

impl Stage {
    fn new(cols: i32, rows: i32) -> Stage {
        Stage { cols: cols, rows: rows }
    }

    fn draw_walls(&self) {
        for x in range(0, self.cols) {
            move(0, x);
            printw(WALL);
            move(self.rows, x);
            printw(WALL);
        }

        for y in range(0, self.rows) {
            move(y, 0);
            printw(WALL);
            move(y, self.cols);
            printw(WALL);
        }
    }

    fn center(&self) -> (i32,i32) {
        (self.cols / 2, self.rows / 2)
    }
}

struct Snake {
    pos: Position,
    symbol: &'static str
}

impl Snake {
    fn new (stage: &Stage) -> Snake {
        let (x,y) = stage.center();
        Snake {
            pos: Position { x: x, y: y },
            symbol: "@"
        }
    }

    fn draw (&mut self) {
        move(self.pos.y, self.pos.x);
        printw(self.symbol);
    }
}

fn main () {
    initscr();
    noecho();
    curs_set(CURSOR_INVISIBLE);
    keypad(stdscr, true);

    let stage = Stage::new(43, 31);
    let mut snake = Snake::new(&stage);

    loop {
        stage.draw_walls();
        snake.draw();
        match getch() {
            KEY_UP      => { snake.pos.up();      },
            KEY_DOWN    => { snake.pos.down();    },
            KEY_LEFT    => { snake.pos.left();    },
            KEY_RIGHT   => { snake.pos.right();   },
            KEY_Q       => { break; }
            _ => {}
        }
        clear();
        refresh();
    }

    endwin();
}
