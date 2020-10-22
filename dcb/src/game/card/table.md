The table of all digimon in the game

# Details
At address [0x216d000](Table::START_ADDRESS) of the game file, the card table begins
with a small header of `0xb` and then the table itself.

# Table Layout
The digimon table has a max size of [0x14950](Table::MAX_BYTE_SIZE), but does not
necessary use all of this space, but it does follow this layout:

| Offset | Size     | Type                              | Name                 | Details                                                                 |
| ------ | -------- | --------------------------------- | -------------------- | ----------------------------------------------------------------------- |
| 0x0    | 0x4      | u32                               | Magic                | Always contains the string "0ACD" (= [0x44434130](Table::HEADER_MAGIC)) |
| 0x4    | 0x2      | u16                               | Number of digimon    |                                                                         |
| 0x6    | 0x1      | u8                                | Number of items      |                                                                         |
| 0x7    | 0x1      | u8                                | Number of digivolves |                                                                         |
| 0x8    | variable | [`CardEntry`](#card-entry-layout) | Card Entries         | A contiguous array of [Card Entry](#card-entry-layout)                  |

# Card Entry Layout
Each card entry consists of a header of the card

| Offset | Size     | Type                                 | Name            | Details                                      |
| ------ | -------- | ------------------------------------ | --------------- | -------------------------------------------- |
| 0x0    | 0x3      | [`Card Header`](#card-header-layout) | Card Header     | The card's header                            |
| 0x3    | variable |                                      | Card            | Either a [Digimon], [Item] or [Digivolve]    |
| ...    | 0x1      | u8                                   | Null terminator | A null terminator for the card (must be `0`) |

# Card Header Layout
The card header determines which type of card this card entry has.

| Offset | Size | Type         | Name      | Details                                          |
| ------ | ---- | ------------ | --------- | ------------------------------------------------ |
| 0x0    | 0x2  | u16          | Card id   | This card's ID                                   |
| 0x2    | 0x1  | [`CardType`] | Card type | The card type ([Digimon], [Item] or [Digivolve]) |
