# `tim` layout

This module stores the definition of a `tim` image with the [`Tim`] type.

# Layout

A `tim` image has the following layout:

| Offset | Size                          | Type     | Name               | Details                                                                                        |
| ------ | ----------------------------- | -------- | ------------------ | ---------------------------------------------------------------------------------------------- |
| 0x0    | 0x8                           | Header   | Header             | The [image header](Header)                                                                     |
| 0x8    | 0xc + `num_colors` * 0x2      | [`Clut`] | Color lookup table | The color lookup table for indexed images. Optional in direct-color images.                    |
| ...    | 0xc + `pixels` * `pixel_size` | [`Img`]  | Pixel data         | The image data itself. Size of each pixel can vary depending on the `bbp` found in the header. |