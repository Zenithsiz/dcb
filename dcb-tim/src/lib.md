# `.TIM` File support

This crate provides support for reading and writing `.TIM` files.

# File format

The `.TIM` file format (from here onwards, `tim`) is format that allows
both color-indexed and direct-color images to be stored.

More precisely there are 4 modes which an image can have, two color-indexed modes (`4-bit` and `8-bit` indexes),
as well two direct-color modes (`16-bit` and `24-bit` pixels).

See [`BitsPerPixel`] for more information.

Both indexed modes, as well as `16-bit` support a transparency bit that allows for full opacity or transparency only.

In indexed mode, a color lookup table ([`Clut`]) is also included with the colors (specified in `16-bit` pixels).

This `clut` may have several pallettes, usually by providing a multiple of the number of indexes
supported (16 for 4-bit and 256 for 8-bit indexes).

This is in no way enforced by the file format, and it is up for the implementation to discern how to distribute
the extra colors on indexed images.

# Layout

See the [`tim`] module for more details on how the file is layed out.