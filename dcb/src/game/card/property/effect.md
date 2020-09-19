A digimon's support effect

This module contains the [`Effect`] struct, which describes a support effect.

# Layout
Each support effect has a size of `0x10` bytes, and it's general layout is the following:

| Offset | Size | Type   | Name        | Location | Details                                                |
| ------ | ---- | ------ | ----------- | -------- | ------------------------------------------------------ |
| 0x0    | 0x1  | `bool` | Exists      | N/A      | If `0`, the effect does not exist                      |
| 0x1    | 0x1  | N/A    | Effect Type | N/A      | Determines which [`Effect`] variant is used.           |
| 0x2    | 0xe  | N/A    | Arguments   | N/A      | The arguments used for the current [`Effect`] variant. |
