# Directory entry

Every directory entry must be aligned to a `2048` (`0x800`) byte sector, including
both files and directories, but may span however many sectors it needs (excluding files
which have a max size of ~`4 GiB` and thus a max sector size of `2097152` sectors).

# Layout

Each directory entry has the following layout:

| Offset | Size | Type         | Name                                                                   |
| ------ | ---- | ------------ | ---------------------------------------------------------------------- |
| 0x0    | 0x1  | `Kind`       | Entry kind, `0x0` for none, `0x1` for files and `0x80` for directories |
| 0x1    | 0x3  | `[u8; 3]`    | (Files only) File extension                                            |
| 0x4    | 0x4  | `u32`        | Entry sector position                                                  |
| 0x8    | 0x4  | `u32`        | (Files only) File size in bytes                                        |
| 0xc    | 0x4  | `u32`        | Entry date (seconds since epoch)                                       |
| 0x10   | 0x10 | `[u8; 0x10]` | Entry name                                                             |

# Limitations

The max size for a file is `u32::MAX` bytes, but the filesystem itself can hold `8 TiB` (`2^43` bytes) in total, as each
directory and file position is identified by sector.
