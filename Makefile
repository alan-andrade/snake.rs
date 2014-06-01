.PHONY: all

all: .ncurses .snake

.ncurses:
	make -C ncurses-rs

.snake:
	rustc snake.rs -L ncurses-rs/lib --out-dir bin

test:
	rustc snake.rs -L ncurses-rs/lib --out-dir bin --test
	./bin/snake


update:
	git submodule foreach git pull origin master
