Header for the digimon table.

# Details
The header contains a magic number to identify the table, as well as the number of each card
available.

The number of digimons is measured with a `u16`, but the items and digivolves are only measured using
a `u8`, meaning they have a limit of `256`.

| Offset | Size | Type | Name                 | Details                                                    |
| ------ | ---- | ---- | -------------------- | ---------------------------------------------------------- |
| 0x0    | 0x4  | u32  | Magic                | Always contains the string ["0ACD"]((Table::HEADER_MAGIC)) |
| 0x4    | 0x2  | u16  | Number of digimon    |                                                            |
| 0x6    | 0x1  | u8   | Number of items      |                                                            |
| 0x7    | 0x1  | u8   | Number of digivolves |                                                            |
