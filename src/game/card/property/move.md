A digimon's move

This module contains the [`Move`] struct, which describes a generic move over the triangle, circle or cross.

# Layout
Each move has a size of `0x1c` bytes, and it's layout is the following:

| Offset | Size | Type           | Name    | Location  | Details                           |
| ------ | ---- | -------------- | ------- | --------- | --------------------------------- |
| 0x0    | 0x2  | `u16`          | Power   | `power`   |                                   |
| 0x2    | 0x4  | `u32`          | Unknown | `unknown` | Most likely stores animation data |
| 0x6    | 0x16 | `[char; 0x16]` | Name    | `name`    | Null-terminated                   |
