# `Drv` filesystem

The `Drv` filesystem is the second-stage filesystem used by `dcb` after the standard
`ISO 9660` found in the cd-rom itself.

It's a multi-level read-only filesystem with support for files and directories.

Each directory entry may be either a file or directory and is described by
a `16` byte name and a `UNIX` timestamp (32-bit).

Files are further described by a `3` byte file extension and a `32` bit file
size, thus limiting them to about the `4 GiB` mark.

Directories may have any number of entries, limited just by the total number of sectors
within the drive, around the `8 TiB` mark.

Every directory entry must be aligned to a `2048` (`0x800`) byte sector, including
both files and directories, but may span however many sectors it needs (excluding files
which have a max size of ~`4 GiB` and thus a max sector size of `2097152` sectors).

# Layout

The `.DRV` filesystem splits the file into sectors of `2048` (`0x800`) bytes, dedicating each one
to either a file or directory.
Both files and directories may only begin at the start of each sector, but may span various
sectors.

The filesystem begins with a directory sector, called the 'root' directory.

See the [`dir`] documentation for more information on how each directory works.
