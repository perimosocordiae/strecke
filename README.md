# strecke

Game details:

 - 6x6 board, with 35 total tiles (one space always empty)
 - tiles have 2 ports/side, for 8 ports total
 - may just want to enumerate all types of tiles:
   - see https://felleisen.org/matthias/4500-f19/tiles.html
 - 3 rotations per tile
 - 2-8 players
 - player tokens start on the edge of the board
 - play must advance your own token

Core Design:

 - board + tile + token representations
 - placement evaluator (check for legality + resolve motions)
 - game state manager (track turns, tile availability, game end)
 - AI agents

UI Design:

 - web UI for game state display (from JSON repr)
 - game config + start + play UX

