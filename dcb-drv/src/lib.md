# `drv` filesystem

The `drv` filesystem is a `worm` (write-one-read-many) filesystem,
with support for files and directories.

Each directory is a list of entries, see [`entry`](crate::entry) for more details.

# Layout

The `.DRV` filesystem splits the file into sectors of `2048` (`0x800`) bytes, dedicating each one
to either a file or directory.
Both files and directories may only begin at the start of each sector, but may span various
sectors.

The filesystem begins with a directory sector, called the 'root' directory.

See the [`dir`] documentation for more information on how each directory works.

# Name

`drv` likely stands for 'Drive'.

This is supported by it's usage a drive, e.g. 'B:\\CARD2.CDD' where
the file `B.DRV` exists in the upper `Iso9660` filesystem.
