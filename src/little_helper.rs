use crate::SectionMagic;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{io, io::Cursor, io::Read, io::Write};

/// Helper to assist with reading and writing little-endian values.
pub struct LittleHelper(pub Cursor<Vec<u8>>);
impl LittleHelper {
    /// Creates a new LittleHelper with an empty Vec<u8>.
    pub fn new() -> Self {
        LittleHelper(Cursor::new(Vec::new()))
    }

    /// Returns the inner vector represented by this LittleHelper.
    pub fn contents(self) -> Vec<u8> {
        self.0.into_inner()
    }

    /// Seeks to the given u32.
    pub fn seek_to_u32(&mut self, offset: u32) {
        self.0.set_position(offset as u64);
    }

    /// Returns the current cursor's position as a u32.
    pub fn pos_as_u32(&mut self) -> u32 {
        self.0.position() as u32
    }

    /// Reads a singular byte.
    pub fn read_u8(&mut self) -> Result<u8, io::Error> {
        self.0.read_u8()
    }

    /// Reads a little-endian u16.
    pub fn read_u16_le(&mut self) -> Result<u16, io::Error> {
        self.0.read_u16::<LittleEndian>()
    }

    /// Reads a little-endian u32.
    pub fn read_u32_le(&mut self) -> Result<u32, io::Error> {
        self.0.read_u32::<LittleEndian>()
    }

    /// Reads an arbitrary length of bytes.
    pub fn read_length(&mut self, length: u32) -> Result<Vec<u8>, io::Error> {
        let mut raw_data: Vec<u8> = vec![0; length as usize];
        self.0.read_exact(&mut raw_data)?;
        Ok(raw_data)
    }

    /// Reads our little-endian magic (that is, 4 bytes of u8).
    pub fn read_magic(&mut self) -> Result<SectionMagic, io::Error> {
        let mut raw_magic: SectionMagic = [0, 0, 0, 0];
        self.0.read_exact(&mut raw_magic)?;
        Ok(raw_magic)
    }

    /// Writes a little-endian u32.
    pub fn write_u32_le(&mut self, value: u32) -> Result<(), io::Error> {
        self.0.write_u32::<LittleEndian>(value)
    }

    /// Writes an arbitrary length of bytes.
    pub fn write_length(&mut self, raw_data: &[u8]) -> Result<(), io::Error> {
        self.0.write_all(raw_data)
    }

    /// Writes our little-endian magic (that is, 4 bytes of u8).
    pub fn write_magic(&mut self, magic: SectionMagic) -> Result<(), io::Error> {
        self.0.write_all(&magic)
    }

    /// Writes padding to align this to 4 bytes.
    pub fn write_padding(&mut self, length: u32) -> Result<(), io::Error> {
        let alignment = 4;
        let padding_length: u32 = alignment - (length % alignment);
        if padding_length == alignment {
            // No padding is necessary.
            return Ok(());
        }

        let padding_data: Vec<u8> = vec![0; padding_length as usize];
        self.write_length(&padding_data)
    }
}
