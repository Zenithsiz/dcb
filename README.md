# Dcb

Rust API for the PS1 game [`Digimon Digital Card Battle`](https://en.wikipedia.org/wiki/Digimon_Digital_Card_Battle).

This API offers various entry points to modify the game file, in the `.bin` format, alongside a `.cue`.

## Features

Currently supported features include:

- Per-card editing.
  
  Each card may be edited, including digimon, items and digivolves.
  Currently cards cannot be removed or edited, just added, however.

## Planned features

- Deck manipulation.
  
  Currently half-implemented.
  
  Would allow editing decks of opponents.
- Save file modification.
  
  Would allow the save files to be changed, such as player-decks, progress, cards unlocked, etc.
