# stupid_gifs
A gif player, written in Rust using the [pixels crate](https://docs.rs/pixels/latest/pixels/). 

Simply runs a gif as fluidly as possible without dropping frames, even at the minimum frame time (1/100 second).

## Running:
Just run `cargo build --release` in the repo after cloning, and you'll get a file selection window. Choose a gif, and it should play!

## Keybinds: 
```
Space:      Pause
Left/Right: Skip 5 seconds
Scroll:     Scrub through frames
```
