# CD-ROM/XA Implementation

This module implements the `CD-ROM/XA Mode 2 Form 1` specification
within the [`CdRom`] struct, which takes an arbitrary reader and
allows access to all of it's sectors.

# Layout
The `CD-ROM/XA Mode 2 Form 1` specification dictates that the
file be split into sectors of size `0x930` bytes. See the [`sector`]
module for it's layout.

The file is expected to have a size multiple of `0x930`. The current
implementation returns an error if unable to read the whole `0x930`
bytes, regardless of how many were correctly read.
