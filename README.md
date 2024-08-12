# Conway's Game of Life

Conway's Game of Life written in [Rust](https://www.rust-lang.org/), using the [Bevy](https://bevyengine.org/) game
engine.

Available on [itch.io](https://mnemotic.itch.io/game-of-life).

## Features

Implemented and planned features.

- [X] Pause / unpause the simulation.
- [X] Advance and rewind the simulation a single tick (generation).
- [ ] Basic world editing.
    - [X] Toggle a single cell (alive / dead).
    - [ ] Toggle a rectangular group of cells.
- [X] Increase / decrease simulation rate (speed).
- [ ] Save / load.
- [ ] Zoom.
- [ ] GUI.
    - [ ] World, cell, and simulation statistics.
    - [X] Visual controls.
- [ ] Advanced editing.
    - [ ] Pattern library.
    - [ ] Undo / redo.

## Controls

| Key          | Action                                             |
|--------------|----------------------------------------------------|
| `Space`, `P` | Pause / unpause the simulation.                    |
| `]`          | Advance the simulation a single tick (generation). |
| `[`          | Rewind the simulation a single tick (generation).  |
| `LMB`        | Toggle cell state.                                 |
| `-`          | Decrease simulation rate (speed).                  |
| `=`          | Increase simulation rate (speed).                  |
