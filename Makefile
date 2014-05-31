.PHONY: all

all: .ncurses .snake

.ncurses:
	make -C ncurses-rs

.snake:
	rustc snake.rs -L ncurses-rs/lib --out-dir bin

update:
	git submodule foreach git pull origin master
