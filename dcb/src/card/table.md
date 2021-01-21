The table of all digimon in the game

# Details
The card table contains a small header detailing how many of each cards exist, following by a contiguous
array of card entries.

# Table Layout

| Offset | Size     | Type                              | Name         | Details                                                |
| ------ | -------- | --------------------------------- | ------------ | ------------------------------------------------------ |
| 0x0    | 0x8      | u32                               | Header       | The [table header](Header)                             |
| 0x8    | variable | [`CardEntry`](#card-entry-layout) | Card Entries | A contiguous array of [Card Entry](#card-entry-layout) |

# Card Entry Layout
Each card entry consists of a header of the card, the card itself and a null terminator.

| Offset | Size     | Type           | Name            | Details                                      |
| ------ | -------- | -------------- | --------------- | -------------------------------------------- |
| 0x0    | 0x3      | [`CardHeader`] | Card Header     | The card's header                            |
| 0x3    | variable |                | Card            | Either a [Digimon], [Item] or [Digivolve]    |
| ...    | 0x1      | u8             | Null terminator | A null terminator for the card (must be `0`) |
