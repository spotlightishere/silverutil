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
        enum: resource_types
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
    instances:
      resource_data:
        pos: _root.header.header_length + file_relative_offset
        size: file_size
        type:
          switch-on: _parent.section_magic
          # The constants below are big-endian values.
          # They appear as little-endian within the firmware.
          cases:
            "resource_types::cwbm": cwbm_entry
            "resource_types::sstr": sports_string
            "resource_types::stbm": bitmap_image

            # Generic string types
            "resource_types::acst": str_entry
            "resource_types::str": str_entry
            "resource_types::strt": str_entry
            "resource_types::scst": str_entry

            # TODO: Analyze
            "resource_types::font": generic_data

            # TODO: These appears to contain arrays,
            # with the first value being a u16.
            "resource_types::aali": generic_data
            "resource_types::cevt": generic_data
            "resource_types::colr": generic_data
            "resource_types::sevt": generic_data
            "resource_types::tevt": generic_data

            # TODO: These types are arrays.
            # We should not repeat them between `fixed_array`,
            # but Kaitai lacks generics within types.
            "resource_types::anim": fixed_array
            "resource_types::clov": fixed_array
            "resource_types::csov": fixed_array
            "resource_types::clkh": fixed_array
            "resource_types::pvcd": fixed_array
            "resource_types::pvcl": fixed_array
            "resource_types::sani": fixed_array
            "resource_types::slst": fixed_array
            "resource_types::srvl": fixed_array
            "resource_types::sorc": fixed_array
            "resource_types::sues": fixed_array
            "resource_types::tmlt": fixed_array
            "resource_types::tmap": fixed_array
            "resource_types::tvcs": fixed_array
            "resource_types::tvcl": fixed_array
            "resource_types::vcvs": fixed_array
            "resource_types::vlyt": fixed_array
            "resource_types::vslt": fixed_array
            "resource_types::view": fixed_array

            # TODO: This feels off.
            "resource_types::mast": resource_array

            _: generic_data

  ################
  # Helper types #
  ################

  # Many resource types contain arrays of elements.
  # A fixed array has the first u32 as its element count.
  fixed_array:
    seq:
      - id: array_count
        type: u4
      - id: elements
        type: fixed_array_entry
        repeat: expr
        repeat-expr: array_count

  # Every element of a fixed array begins with
  # one u32 specifying its element size.
  #
  # Often times, another u32 will follow
  # with the resource type magic repeated,
  # but this is not always the case.
  fixed_array_entry:
    seq:
      - id: entry_size
        type: u4
      # Total entry size includes the four bytes.
      # It should be aligned to four.
      - id: element
        size: entry_size + 4
        type:
          # TODO: These types are arrays.
          # We should not repeat them between `fixed_array`,
          # but Kaitai lacks generics within types.
          #
          # We access the parent section for its magic.
          switch-on: _parent._parent._parent.section_magic
          cases:
            "resource_types::pvcd": resource_pairing
            "resource_types::pvcl": resource_pairing
            "resource_types::tmap": resource_pairing
            "resource_types::tvcl": table_view_layout
            "resource_types::tvcs": table_view_unknown

            # TODO: These appear to also contain array types internally.
            "resource_types::clov": clov_entry
            "resource_types::csov": clov_entry

            "resource_types::tmlt": generic_data

            # TODO: These two appear to be highly similar,
            # if not the same.
            "resource_types::vlyt": view_layout
            "resource_types::vslt": generic_data
            "resource_types::view": view

            # TODO
            "resource_types::anim": generic_data
            "resource_types::clkh": generic_data
            "resource_types::slst": generic_data
            "resource_types::srvl": generic_data

  # There are also "resource arrays".
  # Unlike a fixed array, each element entry
  # additionally contains an inner array.
  resource_array:
    seq:
      - id: array_count
        type: u4
      - id: elements
        type: resource_array_entry
        repeat: expr
        repeat-expr: array_count

  resource_array_entry:
    seq:
      - id: inner_count
        type: u4
      - id: inner_elements
        type: resource_pairing
        repeat: expr
        repeat-expr: inner_count

  # A pairing of a section type to a resource ID.
  resource_pairing:
    seq:
      - id: section_type
        type: u4
        enum: resource_types
      - id: resource_id
        type: u4

  generic_data:
    seq:
      - id: contents
        size-eos: true

  ##########
  # Images #
  ##########
  bitmap_image:
    seq:
      - id: image_type
        type: u2
      - id: is_external
        type: u2
      - id: rendered_width
        type: u2
      - id: color_depth
        type: u2
      - id: padding_one
        type: u4
      - id: padding_two
        type: u4
      - id: width
        type: u4
      - id: height
        type: u4
      - id: resource_id
        type: u4
      - id: contents_length
        type: u4
      - id: contents
        size: contents_length

  ########################
  # Resource definitions #
  ########################
  str_entry:
    seq:
      - id: string
        type: strz
        encoding: ascii

  # A reference to SORC data.
  sorc_entry:
    seq:
      # Presumably this element's tag.
      - id: sorc_length
        type: u4
      - id: pairing
        type: resource_pairing

  clov_entry:
    seq:
      - id: unk_resource_id
        type: u4
      - id: unk_count
        type: u4
      - id: unk
        type: sorc_entry
        repeat: expr
        repeat-expr: unk_count

  # An array of resources.
  # Note: Its length appears to vary across versions.
  cwbm_entry:
    seq:
      - id: resources
        type: u4
        repeat: expr
        repeat-expr: _io.size / 4

  sports_string:
    seq:
      # The name of the `.wav` within resources.
      # For example, 1E6B55C95726B4B9.wav.
      - id: filename
        type: u8
      # UISS, as in `rsrc/Speakable/UISS0000`.
      - id: uiss_str
        type: u4
      # Unknown.
      - id: unknown_value
        type: u4

  ##############
  # Table View #
  ##############
  table_view_layout:
    seq:
      - id: section_type
        type: u4
      - id: resource_id_one
        type: u4
      - id: resource_id_two
        type: u4
      # Appears to always be 1.
      - id: unknown
        type: u4

  table_view_unknown:
    seq:
      - id: section_type
        type: u4
      - id: unknown_max_values
        type: s4
        repeat: expr
        repeat-expr: 4
      # Note: this likely isn't actually a pairing.
      - id: resource_ids
        type: resource_pairing
        repeat: expr
        repeat-expr: 5
      - id: zero_one
        type: u4
      - id: zero_two
        type: u4
      # Perhaps attributes? 0x191701 has been observed.
      - id: unknown
        type: u4
      - id: zero_three
        type: u4

  ###############
  # View Layout #
  ###############
  view_layout:
    seq:
      # One ID typically matches between View, VSLT, and VLyt.
      - id: view_resource_id
        type: u4
      - id: inner_count
        type: u4
      - id: inner_array
        type: view_layout_array
        repeat: expr
        repeat-expr: inner_count

  view_layout_array:
    seq:
      - id: size
        type: u4
      # Loose notes:
      # - If a multiple of 0xc, contains three u32s.
      # - If 0x10, generally contains resource ID with SORC reference.
      # Unclear on how this is determined.
      - id: contents
        size: size + 4

  view_layout_inner:
    seq:
      - id: some_id
        type: u4
      - id: zero_one
        type: u4
      - id: zero_two
        type: u4

  #########
  # Views #
  #########
  view:
    seq:
      # One ID typically matches between View, VSLT, and VLyt.
      - id: view_resource_id
        type: u4
      # TODO: Revisit.
      # Perhaps these are positioning values?
      # They often have negative values.
      - id: unknown_zero_values
        type: u4
        repeat: expr
        repeat-expr: 12
      # These have negative values in all observed examples.
      - id: negative_values
        type: u4
        repeat: expr
        repeat-expr: 4
      - id: zero_value
        type: u4
      - id: resource_id
        type: u4
      - id: zero_again
        type: u4
      # The backing view type when rendering.
      - id: view_type
        type: u4
        enum: view_types
enums:
  #################
  # Section types #
  #################

  # The constants below are big-endian values.
  # They appear as little-endian within thie firmware.
  # For example,
  resource_types:
    # 'Str ' (BE) or ' rtS' (LE)
    0x53747220: str
    # ANIM
    0x414E494D: anim
    # AEVT
    0x41455654: aevt
    # AALI
    0x41414C49: aali
    # ACST
    0x41435354: acst
    # CEVT
    0x43455654: cevt
    # CLKH: Clock hand?
    0x434C4B48: clkh
    # CLov
    0x43536F76: clov
    # COLR: Colors
    0x434F4C52: colr
    # CSov
    0x434C6F76: csov
    # CWBM: Clock ? bitmaps?
    0x4357424d: cwbm
    # DECO: Perhaps wallpapers?
    0x4445434F: deco
    # FONT
    0x464F4E54: font
    # LDTm: Date/Time locale
    0x4C44546D: ldtm
    # MASt
    0x4D415374: mast
    # PVCD: PickerView Cell ?
    0x50564344: pvcd
    # PVCL
    0x5056434C: pvcl
    # PVCR
    0x50564352: pcvr
    # SANI
    0x53414E49: sani
    # SCRN
    0x5343524E: scrn
    # SCST: Screen controller strings
    0x53435354: scst
    # SEVT: Screen events?
    0x53455654: sevt
    # SLst: Screen list?
    0x534C7374: slst
    # SORC
    0x534F5243: sorc
    # SRVL
    0x5352564C: srvl
    # SStr: Sports/Workout Strings
    0x53537472: sstr
    # SUes
    0x53557365: sues
    # StBM: Status bar bitmap image
    0x5374424D: stbm
    # StrT: Transition strings
    0x53747254: strt
    # T10N: Translations
    0x5431304E: t10n
    # TEVT
    0x54455654: tevt
    # TMLT
    0x544D4C54: tmlt
    # TMap
    0x544d6170: tmap
    # TVCL: Table View Cell Layout?
    0x5456434C: tvcl
    # TVCS: Table View Cell ?
    0x54564353: tvcs
    # VCvs: View Controller something
    0x56437673: vcvs
    # VLyt: View layout
    0x564C7974: vlyt
    # VSlt: View screen layout?
    0x56536C74: vslt
    # View
    0x56696577: view

  ##############
  # View types #
  ##############

  # Note: this is non-exhaustive.
  # Some types are hardcoded within firmware,
  # and some are present within Silver DBs.
  view_types:
    # bmdg: Bitmap drawing
    0x626D6467: bitmap_digit
    # Button
    0x62627574: button
    # imag: A color image
    0x696D6167: image
    # Text
    0x74657874: text
    # PHTL: Photo view, landscape
    0x5048544c: photo_landscape
    # PHTP: Photo view, portrait
    0x50485450: photo_portrait
    # SSVW: Unknown, "TSSView"
    0x53535657: ss_view
    # abar: Analog bar view
    0x61626172: analog_bar
    # adbm: Analog DB meter
    0x6164626D: analog_db_meter
    # aclk: Analog clock
    0x61636c6b: analog_clock
    # capt: Captions
    0x63617074: caption
    # comp: Composite view
    0x636f6d70: composite
    # grad: Linear gradient view
    0x67726164: linear_gradient
    # lbuf: Live buffer view
    0x6c627566: live_buffer
    # metr: Decibel meter
    0x6d657472: decibel_meter
    # pgin: Page indicator
    0x7067696e: page_indicator
    # pgog: Progress view
    0x70676f67: progress
    # pkrv: Silver Picker view
    0x706b7276: picker_view
    # psts: Power status
    0x70737473: power_status
    # rtba: Radio tuner bar
    0x72746261: radio_tuner
    # sbgv: SpringBoard grid view
    0x73626776: springboard_grid
    # sbtl: Subtitle view
    0x7362746c: subtitles
    # sbit: SpringBoard item view
    0x73626974: springboard_item
    # sbvi: Unknown, TTableViewDataSource
    0x73627669: table_data_source
    # slav: Silver "slide action" view
    0x736c6176: slide_action
    # slid: Slider view
    0x736c6964: slider
    # spok: Possibly VoiceOver related?
    # Appears to be a generic view.
    0x73706f6b: voiceover
    # star: Five-star rating vie
    0x73746172: star_rating_view
    # tabl: Table
    0x7461626c: table
    # tslv: Silver toggle view
    0x74736c76: toggle_view
    # visu: Visual text
    0x76697375: visual_text
