//! Minimal protobuf wire-format reader: just enough for Google's migration payload.

use super::ImportError;

const VARINT_PAYLOAD_MASK: u8 = 0x7f;
const VARINT_CONTINUATION_BIT: u8 = 0x80;
const VARINT_CHUNK_BITS: usize = 7;
const VARINT_MAX_BITS: usize = 64;
const WIRE_TYPE_MASK: u64 = 0b111;
const FIELD_NUMBER_SHIFT: u32 = 3;
const WIRE_TYPE_VARINT: u64 = 0;
const WIRE_TYPE_LENGTH_DELIMITED: u64 = 2;

pub enum Field<'a> {
    Varint(u64),
    Bytes(&'a [u8]),
}

pub struct Reader<'a> {
    remaining: &'a [u8],
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { remaining: bytes }
    }

    /// Next `(field number, value)`, or `None` at the end of the message.
    pub fn next_field(&mut self) -> Result<Option<(u64, Field<'a>)>, ImportError> {
        if self.remaining.is_empty() {
            return Ok(None);
        }
        let tag = self.read_varint()?;
        let field = match tag & WIRE_TYPE_MASK {
            WIRE_TYPE_VARINT => Field::Varint(self.read_varint()?),
            WIRE_TYPE_LENGTH_DELIMITED => Field::Bytes(self.read_length_delimited()?),
            _ => return Err(ImportError::BadPayload),
        };
        Ok(Some((tag >> FIELD_NUMBER_SHIFT, field)))
    }

    fn read_varint(&mut self) -> Result<u64, ImportError> {
        let mut value = 0u64;
        for shift in (0..VARINT_MAX_BITS).step_by(VARINT_CHUNK_BITS) {
            let byte = self.take_byte()?;
            value |= u64::from(byte & VARINT_PAYLOAD_MASK) << shift;
            if byte & VARINT_CONTINUATION_BIT == 0 {
                return Ok(value);
            }
        }
        Err(ImportError::BadPayload)
    }

    fn read_length_delimited(&mut self) -> Result<&'a [u8], ImportError> {
        let length = usize::try_from(self.read_varint()?).map_err(|_| ImportError::BadPayload)?;
        if length > self.remaining.len() {
            return Err(ImportError::BadPayload);
        }
        let (bytes, rest) = self.remaining.split_at(length);
        self.remaining = rest;
        Ok(bytes)
    }

    fn take_byte(&mut self) -> Result<u8, ImportError> {
        let (&byte, rest) = self.remaining.split_first().ok_or(ImportError::BadPayload)?;
        self.remaining = rest;
        Ok(byte)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_multi_byte_varints() {
        // field 1, varint 300 (0xAC 0x02)
        let mut reader = Reader::new(&[0x08, 0xAC, 0x02]);
        let (number, field) = reader.next_field().unwrap().unwrap();
        assert_eq!(number, 1);
        assert!(matches!(field, Field::Varint(300)));
        assert!(reader.next_field().unwrap().is_none());
    }

    #[test]
    fn reads_length_delimited_fields() {
        let mut reader = Reader::new(&[0x0a, 0x02, b'h', b'i']);
        let (_, field) = reader.next_field().unwrap().unwrap();
        assert!(matches!(field, Field::Bytes(b"hi")));
    }

    #[test]
    fn rejects_truncated_and_unsupported_input() {
        assert!(Reader::new(&[0x0a, 0x05, 1]).next_field().is_err());
        assert!(Reader::new(&[0x08]).next_field().is_err());
        assert!(Reader::new(&[0x0d, 0, 0, 0, 0]).next_field().is_err());
        assert!(Reader::new(&[0xff; 11]).next_field().is_err());
    }
}
