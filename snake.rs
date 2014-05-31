#![feature(globs)]

extern crate ncurses;
use ncurses::*;

static KEY_Q: i32 = 'q' as i32;

enum Direction {
    Up, Down, Left, Right
}

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

fn main () {
    initscr();
    noecho();
    curs_set(CURSOR_INVISIBLE);
    keypad(stdscr, true);

    let mut pos = Position{ x:0, y:0 };

    loop {
        match getch() {
            KEY_UP      => { pos.up();      },
            KEY_DOWN    => { pos.down();    },
            KEY_LEFT    => { pos.left();    },
            KEY_RIGHT   => { pos.right();   },
            KEY_Q       => { break; }
            _ => {}
        }
        clear();
        move(pos.y, pos.x);
        printw("#");
        refresh();
    }

    endwin();
}
