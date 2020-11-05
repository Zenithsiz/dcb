Header for each card in the table.

# Details
Each header contains the id of the card, as well as it's type.

| Offset | Size | Type         | Name      | Details                                          |
| ------ | ---- | ------------ | --------- | ------------------------------------------------ |
| 0x0    | 0x2  | u16          | Card id   | This card's ID                                   |
| 0x2    | 0x1  | [`CardType`] | Card type | The card type ([Digimon], [Item] or [Digivolve]) |
