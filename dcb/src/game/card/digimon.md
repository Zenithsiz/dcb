A digimon card

This module contains the [`Digimon`] struct, which describes a digimon card.

# Layout
The digimon card has a size of `0x138` bytes, and it's layout is the following:

| Offset | Size | Type                | Name                     | Location               | Details                                                                             |
| ------ | ---- | ------------------- | ------------------------ | ---------------------- | ----------------------------------------------------------------------------------- |
| 0x0    | 0x15 | `[u8; 0x15]`        | Name                     | `name`                 | Null-terminated                                                                     |
| 0x15   | 0x2  | `u16`               | Unknown                  | `unknown_15`           |                                                                                     |
| 0x17   | 0x1  | `u8`                | Speciality & Level       | `speciality level`     | The bottom nibble of this byte is the level, while the top nibble is the speciality |
| 0x18   | 0x1  | `u8`                | DP                       | `dp_cost`              |                                                                                     |
| 0x19   | 0x1  | `u8`                | +P                       | `dp_give`              |                                                                                     |
| 0x1a   | 0x1  | `u8`                | Unknown                  | `unknown_1a`           | Is `0` for all digimon                                                              |
| 0x1b   | 0x2  | `u16`               | Health                   | `hp`                   |                                                                                     |
| 0x1d   | 0x1c | [`Move`]            | Circle Move              | `move_circle`          |                                                                                     |
| 0x39   | 0x1c | [`Move`]            | Triangle move            | `move_triangle`        |                                                                                     |
| 0x55   | 0x1c | [`Move`]            | Cross move               | `move_cross`           |                                                                                     |
| 0x71   | 0x20 | [`EffectCondition`] | First condition          | `effect_conditions[0]` |                                                                                     |
| 0x91   | 0x20 | [`EffectCondition`] | Second condition         | `effect_conditions[1]` |                                                                                     |
| 0xb1   | 0x10 | [`Effect`]          | First effect             | `effects[0]`           |                                                                                     |
| 0xc1   | 0x10 | [`Effect`]          | Second effect            | `effects[1]`           |                                                                                     |
| 0xd1   | 0x10 | [`Effect`]          | Third effect             | `effects[2]`           |                                                                                     |
| 0xe1   | 0x1  | [`CrossMoveEffect`] | Cross move effect        | `cross_move_effect`    |                                                                                     |
| 0xe2   | 0x1  | `u8`                | Unknown                  | `unknown_e2`           |                                                                                     |
| 0xe3   | 0x1  | [`ArrowColor`]      | Effect arrow color       | `effect_arrow_color`   |                                                                                     |
| 0xe4   | 0x54 | `[[u8; 0x15]; 4]`   | Effect description lines | `effect_description`   | Each line is` 0x15` bytes, split over 4 lines, each null terminated                 |
