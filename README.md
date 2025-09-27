# file-explorer-tui

<img width="1478" height="996" alt="Screenshot_2025-09-27_21-20-31" src="https://github.com/user-attachments/assets/0612cf8a-3954-4651-b8cf-5923be13117d" />

### How to run

- clone the repo
- navigate into it
- run `cargo run` in your terminal of choice

### Why?

mainly to
- learn rust
- apply learnings from reading the rust-book
- and of course: HAVE FUN (:

### Features

- Live reload when files are created/deleted via notify crate
- Cheatsheet to see all keybinds (press c)
- Create files
- Delete files
- Rename files
- Open files with system-provided program
- Bulk delete files by adding them into the "Selected files" stack
- Remembers in which directory you went into, for each directory (currently only in-memory, e.g. on an "app-running" basis)
- Toggle selected files window
- Cross-platform (not tested on windows lol)
