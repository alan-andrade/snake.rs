#![feature(globs)]

extern crate ncurses;
use ncurses::*;

static KEY_Q: i32 = 'q' as i32;

fn main () {
    initscr();
    noecho();
    keypad(stdscr, true);

    loop {
        match getch() {
            KEY_UP      => { printw("up"); },
            KEY_DOWN    => { printw("down"); },
            KEY_LEFT    => { printw("left"); },
            KEY_RIGHT   => { printw("right"); },
            KEY_Q       => { break; }
            _ => {}
        }
    }

    endwin();
}
