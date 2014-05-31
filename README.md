# Snake.rs

This is the tipical snake game implemented in Rust.

# Install

First get the latest version of rust from rust-lang.org

```bash
git clone git@github.com:alan-andrade/snake.rs.git
cd snake.rs
git submodule init
git submodule update
make
./bin/snake
```

If compilation fails, update ncurses-rs with this:
`git submodule foreach git pull origin master`
