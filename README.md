# silverutil
A utility to read (and, eventually, modify!) the contents of SilverDB files from iPods.

Tested on 5th, 6th, and 7th generation iPod nanos.

## Format
Within the external `rsrc` ("iPod Resources") filesystem, UI translations and date/time locale can be found in `SilverDB.xx_XX.LE.bin`), along with bitmap images in `SilverImagesDB.LE.bin`.
There is an additional database internal to `osos` containing upwards of 40 sections, depending on the version. This contains the default `en_US` translation.

For detailed information into the format of SilverDB files, please see [SilverDB Format](/docs/silverdb_format.md) or its respective [Kaitai Struct](/docs/silverdb.ksy) definition.
