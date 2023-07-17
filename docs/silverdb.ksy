meta:
  id: silverdb
  file-extension: bin
  endian: le
  # Somewhat based off https://pastebin.com/3y4CqSTU.
  # (https://web.archive.org/web/20230715222440/https://pastebin.com/raw/3y4CqSTU)
  xref: https://pastebin.com/3y4CqSTU
seq:
  - id: header
    type: silverdb_header
  - id: sections
    type: section
    repeat: expr
    repeat-expr: header.section_count
types:
  # Consistent for all SilverDB variants - internal OSOS, image, and strings.
  silverdb_header:
    seq:
      # Consistently 0x3.
      - id: version
        type: u4
      # Possibly length related?
      - id: what_is_this
        type: u4
      - id: section_count
        type: u4
  section:
    seq:
      - id: header
        type: section_header
  section_header:
    seq:
      - id: section_magic
        type: u4
      - id: section_file_count
        type: u4
      # Possibly flags?
      - id: further_unknown
        type: u4
      # Multiply by 4 to get the offset relative to the header.
      - id: section_offset
        type: u4
    instances:
      resource_entries:
        type: resource_entry_metadata
        pos: section_offset
        repeat: expr
        repeat-expr: section_file_count
  resource_entry_metadata:
    seq:
      - id: file_id
        type: u4
      - id: file_relative_offset
        type: u4
      - id: file_size
        type: u4
