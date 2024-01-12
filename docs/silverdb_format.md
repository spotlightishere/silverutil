# SilverDB Format
"Silver" is the codename of much of the iPod's UI framework. Every SilverDB database contains multiple types of resources, henceforth referenced as "sections". Every section contains multiple of a resource, referred to as "content". You identify content based on its ID - for example, `0x0dad06d8`.

(This may or may not be the correct terminology used, but it seems close enough.)


## File Format
All known databases have a version of `3`. Immediately following the header, section metadata is present. All data is little endian.

Although sections define an offset to resource metadata, resource metadata is immediately followed by section metadata.

| Offset | Type | Description                             |
|--------|------|-----------------------------------------|
| 0x0    | [Database Header](#database-header)  | Defines section count. |
| 0xc    | [Section Metadata](#section-metadata) | Repeats for as many sections as defined. |
| ...    | [Resource metadata](#resource-metadata) | Repeats for as many resource entries defined in sections. |
| ...    | Resource contents  | All data offsets are relative to this. |

### Database Header
All known firmware versions ensure a database version of `3`.

| Offset | Type | Description   |
|--------|------|---------------|
| 0x0    | u32  | Version       |
| 0x4    | u32  | Header length |
| 0x8    | u32  | Section count |

### Section Metadata
| Offset | Type | Description                                           |
|--------|------|-------------------------------------------------------|
| 0x0    | u32  | [Section type](#section-types).                       |
| 0x4    | u32  | Amount of resource entries this section contains      |
| 0x8    | u32  | Whether resource IDs jump around, or increase by one. |
| 0xc    | u32  | Offset to where this section's [resource metadata](#resource-metadata) array begins, relative to the file's start (0x0). |

### Resource Metadata
| Offset | Type | Description                  |
|--------|------|------------------------------|
| 0x0    | u32  | ID of this resource          |
| 0x4    | u32  | Offset to this resource's data, relative to file contents (proceeding resource metadata entries) |
| 0x8    | u32  | Resource size                |

## Section types
This section should not be considered exhaustive, as its information is largely focused on the 5th generation iPod nano.
Some descriptions may be wrong - please feel free to create a pull request and expand or elaborate on section types!

Some section types have contents which represent an array of elements within. In these situations, the first two bytes (a `uint16_t`) represent the amount of elements within.

Natively, section types are stored in little-endian format.
The names of sections are presented in big-endian for readability (i.e. `BMap` is `paMB` in firmware).

| Name   | Magic        | Description |
|--------|--------------|-------------|
| `AALI` | `0x41414c49` |  |
| `ACST` | `0x41435354` | Possibly "animation controller string", as contents have names similar to `TPhotosAppCntlr`.  |
| `AEVT` | `0x41455654` |  |
| `ANIM` | `0x414e494d` |  |
| `BMap` | `0x70614d42` | Bitmap imagery, commonly found in `SilverImagesDB.LE.bin`. |
| `CEVT` | `0x43455654` |  |
| `CLov` | `0x434c6f76` |  |
| `COLR` | `0x434f4c52` | Defines colors used in UI elements. |
| `CLKH` | `0x434c4b48` | Seen within an iPod nano 7G's internal ROM. |
| `CSov` | `0x43536f76` |  |
| `CWBM` | `0x4357424d` | Seen within an iPod nano 7G's internal ROM. |
| `DECO` | `0x4445434f` | Seen within an iPod nano 7G's internal ROM. |
| `EEEE` | `0x45454545` |  |
| `FONT` | `0x464f4e54` |  |
| `ITEM` | `0x4954454d` |  |
| `LDTm` | `0x4c44546d` | Referenced as `TLocaleDateTimeResource` within firmware. |
| `MASt` | `0x4d415374` |  |
| `PVCD` | `0x50564344` | Seen within an iPod nano 7G's internal ROM. |
| `PVCL` | `0x5056434c` | Seen within an iPod nano 7G's internal ROM. |
| `PVCR` | `0x50564352` | Seen within an iPod nano 7G's internal ROM. |
| `SANI` | `0x53414e49` |  |
| `SCRN` | `0x5343524e` |  |
| `SCRT` | `0x53435254` |  |
| `SCST` | `0x53435354` | Possibly "Silver/UI controller string", as contents match C++ class names (like `TTrainer_Cntlr_ConfirmationAlert`). |
| `SEVT` | `0x53455654` |  |
| `SLst` | `0x534c7374` |  |
| `SORC` | `0x534f5243` |  |
| `SRVL` | `0x5352564c` |  |
| `SStr` | `0x53537472` |  |
| `SUse` | `0x53557365` |  |
| `StBM` | `0x5374424d` | Seen within an iPod nano 7G's internal ROM. Perhaps bitmap images? |
| `Str ` | `0x53747220` | Strings used within various UI components. Translations can be found within `SilverDB.xx_XX.LE.bin`. |
| `StrT` | `0x53747254` | Base descriptions of view text (i.e. `PhotosSettingsSlideshowMusic_Screen_Nested_Default` or `PhotosGL_Camera_Delete_All_Confirmation_Alt`) |
| `T10N` | `0x5431304e` |  |
| `TEVT` | `0x54455654` |  |
| `TLIP` | `0x544c4950` |  |
| `TLOP` | `0x544c4f50` |  |
| `TMLT` | `0x544d4c54` |  |
| `TMap` | `0x544d6170` | Seen within an iPod nano 7G's internal ROM. |
| `TVCL` | `0x5456434c` | Table view cell? |
| `TVCS` | `0x54564353` | Table view cell "from Silver"? |
| `TrIN` | `0x5472494e` |  |
| `TrIO` | `0x5472494f` |  |
| `TrOO` | `0x54724f4f` |  |
| `TrOU` | `0x54724f55` |  |
| `VCvs` | `0x56437673` |  |
| `VLyt` | `0x564c7974` |  |
| `VSlt` | `0x56536c74` |  |
| `View` | `0x56696577` |  |

TODO: What's `StSt`, and how is it utilized? It's referenced in a few areas, yet seemingly not present.
