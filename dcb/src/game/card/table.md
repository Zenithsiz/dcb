The table of all digimon in the game

# Details
The card table begins at address [0x216d000](Table::START_ADDRESS) of the game file,
containing a header with the number of each card, and then a contiguous array of card entries.

# Table Layout
The digimon table has a max size of [0x14950](Table::MAX_BYTE_SIZE), but does not
necessary use all of this space, but it does follow this layout:

| Offset | Size     | Type                              | Name         | Details                                                |
| ------ | -------- | --------------------------------- | ------------ | ------------------------------------------------------ |
| 0x0    | 0x8      | u32                               | Header       | The [table header](TableHeader)                        |
| 0x8    | variable | [`CardEntry`](#card-entry-layout) | Card Entries | A contiguous array of [Card Entry](#card-entry-layout) |

# Card Entry Layout
Each card entry consists of a header of the card

| Offset | Size     | Type           | Name            | Details                                      |
| ------ | -------- | -------------- | --------------- | -------------------------------------------- |
| 0x0    | 0x3      | [`CardHeader`] | Card Header     | The card's header                            |
| 0x3    | variable |                | Card            | Either a [Digimon], [Item] or [Digivolve]    |
| ...    | 0x1      | u8             | Null terminator | A null terminator for the card (must be `0`) |
