An item card

This module contains the [`Item`] struct, which describes an item card.

# Layout
The item card has a size of `0xde` bytes, and it's layout is the following:

| Offset | Size | Type                | Name                     | Location               | Details                                                             |
| ------ | ---- | ------------------- | ------------------------ | ---------------------- | ------------------------------------------------------------------- |
| 0x0    | 0x15 | `[u8; 0x15]`        | Name                     | `name`                 | Null-terminated                                                     |
| 0x15   | 0x4  | `u32`               | Unknown                  | `unknown_15`           |                                                                     |
| 0x19   | 0x20 | [`EffectCondition`] | First condition          | `effect_conditions[0]` |                                                                     |
| 0x39   | 0x20 | [`EffectCondition`] | Second condition         | `effect_conditions[1]` |                                                                     |
| 0x59   | 0x10 | [`Effect`]          | First effect             | `effects[0]`           |                                                                     |
| 0x69   | 0x10 | [`Effect`]          | Second effect            | `effects[1]`           |                                                                     |
| 0x79   | 0x10 | [`Effect`]          | Third effect             | `effects[2]`           |                                                                     |
| 0x89   | 0x1  | [`ArrowColor`]      | Effect arrow color       | `effect_arrow_color`   |                                                                     |
| 0x8a   | 0x54 | `[[u8; 0x15]; 4]`   | Effect description lines | `effect_description`   | Each line is` 0x15` bytes, split over 4 lines, each null terminated |
