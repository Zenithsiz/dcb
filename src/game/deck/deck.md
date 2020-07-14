A deck

This module contains the [`Deck`] struct, which describes a deck.

# Layout
Each deck has a size of `0x6e` bytes, and it's layout is the following:

| Offset | Size | Type         | Name       | Location     | Details              |
| ------ | ---- | ------------ | ---------- | ------------ | -------------------- |
| 0x0    | 0x3c | `[u16; 30]`  | Cards      | `cards`      | List of all card ids |
| 0x3c   | 0x13 | `[u8; 0x13]` | Deck name  | `name`       | Null terminated      |
| 0x4f   | 0x13 | `[u8; 0x13]` | Owner name | `owner`      | Null terminated      |
| 0x62   | 0x13 | `[u8; 0xc]`  | Unknown    | `unknown_62` |                      |
