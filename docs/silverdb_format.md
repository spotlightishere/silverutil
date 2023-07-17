# SilverDB Format
Every SilverDB database contains multiple types of resources, henceforth referenced as "sections". Every section contains multiple of a resource, referred to as "content". You identify content based on its ID - for example, `0x0dad06d8`.

(This may or may not be the naming internally used, but it seems close enough.)


## File Format
All known databases have a version of `3`. Immediately following the header, section metadata is present. All data is little endian.

Although sections define an offset to resource metadata, resource metadata is immediately followed by section metadata.

| Offset | Type | Description                             |
|--------|------|-----------------------------------------|
| 0x0    | [Database Header](#database-header)  | Defines section count. |
| 0xc    | [Section Metadata](#section-metadata) | Repeats for as many sections as defined. |
| ...    | [Resource metadata](#resource-metadata) | Repeats for as many resource entries defined in sections. |
| ...    | Resource contents  | All data offsets base off of this. |

## Database Header
All known databases have a version of `3`.

| Offset | Type | Description                       |
|--------|------|-----------------------------------|
| 0x0    | u32  | Version                           |
| 0x4    | u32  | Unknown. Possibly length related? |
| 0x8    | u32  | Section count                     |

## Section Metadata
| Offset | Type | Description                             |
|--------|------|-----------------------------------------|
| 0x0    | u32  | [Section type](#section-types).         |
| 0x4    | u32  | Amount of resource entries this section contains |
| 0x8    | u32  | Unknown. Possibly related to flags.     |
| 0xc    | u32  | Offset to where this section's [resource metadata](#resource-metadata) array begins, relative to the file's start (0x0). |

## Section types
This section should not be considered exhaustive, as its information is largely focused on the 5th generation iPod nano.
Some descriptions may be wrong - please feel free to submit a pull request and expand/elaborate!

Natively, section types are stored in little-endian format.
The names of sections are presented in big-endian for readability (i.e. `BMap` is `paMB` in firmware).

| Name   | Magic        | Description |
|--------|--------------|-------------|
| `AALI` | `0x41414c49` |  |
| `ACST` | `0x41435354` |  |
| `AEVT` | `0x41455654` |  |
| `ANIM` | `0x414e494d` |  |
| `BMap` | `0x70614d42` | Bitmap imagery, commonly found in `SilverImagesDB.LE.bin`. |
| `CEVT` | `0x43455654` |  |
| `CLov` | `0x434c6f76` |  |
| `COLR` | `0x434f4c52` |  |
| `CSov` | `0x43536f76` |  |
| `EEEE` | `0x45454545` |  |
| `FONT` | `0x464f4e54` |  |
| `ITEM` | `0x4954454d` |  |
| `LDTm` | `0x4c44546d` | Referenced as `TLocaleDateTimeResource` within firmware. |
| `MASt` | `0x4d415374` |  |
| `SANI` | `0x53414e49` |  |
| `SCRN` | `0x5343524e` |  |
| `SCRT` | `0x53435254` |  |
| `SCST` | `0x53435354` |  |
| `SEVT` | `0x53455654` |  |
| `SLst` | `0x534c7374` |  |
| `SORC` | `0x534f5243` |  |
| `SRVL` | `0x5352564c` |  |
| `SStr` | `0x53537472` |  |
| `SUse` | `0x53557365` |  |
| `Str ` | `0x53747220` | Strings used within various UI components. Translations can be found within `SilverDB.xx_XX.LE.bin`. |
| `StrT` | `0x53747254` | Base descriptions of view text (i.e. `PhotosSettingsSlideshowMusic_Screen_Nested_Default` or `PhotosGL_Camera_Delete_All_Confirmation_Alt`) |
| `T10N` | `0x5431304e` |  |
| `TEVT` | `0x54455654` |  |
| `TLIP` | `0x544c4950` |  |
| `TLOP` | `0x544c4f50` |  |
| `TMLT` | `0x544d4c54` |  |
| `TVCL` | `0x5456434c` |  |
| `TVCS` | `0x54564353` |  |
| `TrIN` | `0x5472494e` |  |
| `TrIO` | `0x5472494f` |  |
| `TrOO` | `0x54724f4f` |  |
| `TrOU` | `0x54724f55` |  |
| `VCvs` | `0x56437673` |  |
| `VLyt` | `0x564c7974` |  |
| `VSlt` | `0x56536c74` |  |
| `View` | `0x56696577` |  |

## Resource Metadata
| Offset | Type | Description                  |
|--------|------|------------------------------|
| 0x0    | u32  | ID of this resource          |
| 0x4    | u32  | Offset to this resource's data, relative to file contents (proceeding resource metadata entries) |
| 0x8    | u32  | Resource size                |
