A digimon's effect condition

This module contains the [`EffectCondition`] struct, which describes a condition for an effect.

# Layout
Each support condition has a size of `0x20` bytes, and it's layout is the following:

| Offset | Size | Type                         | Name              | Location       | Details                                                                            |
| ------ | ---- | ---------------------------- | ----------------- | -------------- | ---------------------------------------------------------------------------------- |
| 0x0    | 0x1  | `bool`                       | Misfire           | `misfire`      | If the condition throws a misfire when false                                       |
| 0x1    | 0x1  | `u8`                         |                   | `unknown_1`    | Always zero                                                                        |
| 0x2    | 0x1  | [`DigimonProperty`]          | Property compare  | `property_cmp` | The property to compare to for the condition (or 0 if the condition doesn't exist) |
| 0x3    | 0x5  | `[u8; 0x5]`                  |                   | `unknown_3`    | Unknown                                                                            |
| 0x8    | 0x1  | `DigimonProperty`            | Property argument | `arg_property` | Property argument for the comparison                                               |
| 0x9    | 0xb  | `[u8; 0xb]`                  |                   | `unknown_9`    | Unknown                                                                            |
| 0x14   | 0x2  | `u16`                        | Number argument   | `arg_num`      | Number argument for the comparison                                                 |
| 0x16   | 0x4  | `[u8; 0x4]`                  |                   | `unknown_16`   | Unknown                                                                            |
| 0x1a   | 0x1  | [`EffectConditionOperation`] | Operation         | `operation`    | Operation to use for the comparison                                                |
| 0x1b   | 0x5  | `[u8; 0x5]`                  |                   | `unknown_1b`   | Unknown                                                                            |
