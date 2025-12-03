# strecke

### Game details

 - 6x6 board, with 35 total tiles (one space always empty)
 - tiles have 2 ports/side, for 8 ports total
 - we can enumerate all types of tiles:
   - see https://felleisen.org/matthias/4500-f19/tiles.html
 - 3 rotations per tile
 - 2+ players
 - player tokens start on the edge of the board
 - play must advance your own token
 - last player standing wins

## Agent testing

Show the log of all moves in a single game.

```sh
RUST_LOG=info cargo run --release --example self_play -- --games 1 --agents 0,0,0,1 
```

Run 1000 games and compute statistics of the scores.
Requires running `cargo install xsv` if you don't have it installed already.

```sh
cargo run --release --example self_play -- --games 1000 --agents 0,0,0,1 \
 | xsv stats -n \
 | cut -d, -f4,5,8,9
```
