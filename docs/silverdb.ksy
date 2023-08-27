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
  ############
  # Database #
  ############
  # Consistent for all SilverDB variants - internal OSOS ("ROM"), bitmaps, and strings.
  silverdb_header:
    seq:
      # Consistently 0x3.
      - id: version
        type: u4
      # The length consumed by header content.
      # Resource data begins immediately after all header values.
      - id: header_length
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
      # Whether resource IDs jump around, or increase by one.
      - id: is_sequential
        type: u4
      # Multiply by 4 to get the offset relative to the header.
      - id: section_offset
        type: u4
    instances:
      resource_entries:
        # We need to inform resource entries of our section type
        # in order to be able to parse accordingly.
        type: resource_entry_metadata(section_magic)
        pos: section_offset
        repeat: expr
        repeat-expr: section_file_count
  resource_entry_metadata:
    params:
      - id: resource_type
        type: u4
    seq:
      - id: file_id
        type: u4
      - id: file_relative_offset
        type: u4
      - id: file_size
        type: u4
    instances:
      resource_data:
        pos: _root.header.header_length + file_relative_offset
        size: file_size
        type:
          switch-on: resource_type
          # The constants below are big-endian values.
          # They appear as little-endian within thie firmware.
          cases:
            # 'Str ' (BE) or ' rtS' (LE)
            0x53747220: resource_str
            _: resource_generic_data

  ########################
  # Resource definitions #
  ########################
  resource_str:
    seq:
      - id: string
        type: strz
        encoding: ascii
  resource_generic_data:
    seq:
      - id: contents
        size-eos: true