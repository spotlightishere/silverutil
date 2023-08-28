# silverutil
A utility to read and modify the contents of SilverDB files within iPod firmware. Tested on 5th, 6th, and 7th generation iPod nanos.

## Usage
For up-to-date information, run `silverutil -h`:
```
Usage: silverutil <COMMAND>

Commands:
  extract  Extracts sections within database into a YAML representation
  info     Displays information about contents present within sections
  create   Creates a database from a YAML representation
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Each section is transformed to a separate YAML file. An example of a such a representation is as follows, from the section `StrT` (named `StrT.yaml`):
```yaml
magic: StrT
is_sequential: 1
resources:
- id: 1
  contents: !String orientation.landscape
- id: 2
  contents: !String orientation.alt
# [... continues ...]
```

The special file `metadata.yaml` is used to preserve the order of sections.

## Format
Within the external `rsrc` ("iPod Resources") filesystem, UI translations and date/time locale can be found in `SilverDB.xx_XX.LE.bin`), along with bitmap images in `SilverImagesDB.LE.bin`.
There is an additional database internal to `osos` containing upwards of 40 sections, depending on the version. This contains the default `en_US` translation.

For detailed information into the format of SilverDB files, please see [SilverDB Format](/docs/silverdb_format.md) or its respective [Kaitai Struct](/docs/silverdb.ksy) definition.
