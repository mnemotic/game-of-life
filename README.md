# Conway's Game of Life

Conway's Game of Life in [Rust](https://www.rust-lang.org/), using the [Bevy](https://bevyengine.org/) game engine.

## Features

Implemented and planned features.

- [X] Pause / unpause the simulation.
- [X] Advance and rewind the simulation a single tick (generation).
- [ ] Basic world editing.
    - [ ] Toggle a single cell (alive / dead).
    - [ ] Toggle a rectangular group of cells.
- [ ] Increase / decrease simulation rate (speed).
- [ ] Save / load.
- [ ] GUI.
    - [ ] World, cell, and simulation statistics.
    - [ ] Visual controls.
- [ ] Advanced editing.
    - [ ] Pattern library.
    - [ ] Undo / redo.

## Controls

| Key          | Action                                                            |
|--------------|-------------------------------------------------------------------|
| `Space`, `P` | Pause / unpause the simulation.                                   |
| `]`          | Advance the simulation a single tick (generation) *while paused*. |
| `[`          | Rewind the simulation a single tick (generation) *while paused*.  |
