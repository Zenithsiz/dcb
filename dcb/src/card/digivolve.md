A digivolve card

This module contains the [`Digivolve`] struct, which describes a digivolve card.

# Layout
The digivolve card has a size of `0x6c` bytes, and it's layout is the following:

| Offset | Size | Type              | Name                     | Location             | Details                                                             |
| ------ | ---- | ----------------- | ------------------------ | -------------------- | ------------------------------------------------------------------- |
| 0x0    | 0x15 | `[u8; 0x15]`      | Name                     | `name`               | Null-terminated                                                     |
| 0x15   | 0x3  | `[u8; 3]`         | Unknown                  | `unknown_15`         | Probably contains the card effect                                   |
| 0x8a   | 0x54 | `[[u8; 0x15]; 4]` | Effect description lines | `effect_description` | Each line is `0x15` bytes, split over 4 lines, each null terminated |
