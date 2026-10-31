use std::io::{self, Cursor, Read, Write};

use azalea_buf::{AzBuf, AzBufVar, BufReadError};
use simdnbt::owned::NbtTag;

/// The NBT payload of a custom click action, as vanilla's
/// `ServerboundCustomClickActionPacket.UNTRUSTED_TAG_CODEC` writes it.
///
/// That's `optionalTagCodec(...).apply(lengthPrefixed(65536))`: any unnamed
/// tag (or a lone end tag when there's no payload), with its length in bytes
/// as a VarInt before it. So an empty payload is written as `[1, 0]`, not
/// `[0]`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CustomClickPayload(pub Option<NbtTag>);

/// The most bytes vanilla accepts for a payload.
pub const MAX_CUSTOM_CLICK_PAYLOAD_LENGTH: u32 = 65536;

impl From<NbtTag> for CustomClickPayload {
    fn from(tag: NbtTag) -> Self {
        Self(Some(tag))
    }
}

impl AzBuf for CustomClickPayload {
    fn azalea_read(buf: &mut Cursor<&[u8]>) -> Result<Self, BufReadError> {
        let length = u32::azalea_read_var(buf)?;
        if length > MAX_CUSTOM_CLICK_PAYLOAD_LENGTH {
            return Err(BufReadError::VecLengthTooLong {
                length,
                max_length: MAX_CUSTOM_CLICK_PAYLOAD_LENGTH,
            });
        }
        let mut payload = vec![0; length as usize];
        buf.read_exact(&mut payload)?;
        if payload.is_empty() {
            return Ok(Self(None));
        }
        let tag = simdnbt::owned::read_optional_tag(&mut Cursor::new(payload.as_slice()))
            .map_err(|e| BufReadError::Custom(format!("invalid custom click payload: {e:?}")))?;
        Ok(Self(tag))
    }

    fn azalea_write(&self, buf: &mut impl Write) -> io::Result<()> {
        let mut payload = Vec::new();
        match &self.0 {
            Some(tag) => tag.write(&mut payload),
            // an end tag
            None => payload.push(0),
        }
        (payload.len() as u32).azalea_write_var(buf)?;
        buf.write_all(&payload)
    }
}

#[cfg(test)]
mod tests {
    use simdnbt::owned::NbtCompound;

    use super::*;

    fn round_trip(payload: &CustomClickPayload) -> Vec<u8> {
        let mut bytes = Vec::new();
        payload.azalea_write(&mut bytes).unwrap();
        let mut cursor = Cursor::new(bytes.as_slice());
        assert_eq!(
            &CustomClickPayload::azalea_read(&mut cursor).unwrap(),
            payload
        );
        assert_eq!(cursor.position() as usize, bytes.len());
        bytes
    }

    #[test]
    fn empty_payload_is_a_length_prefixed_end_tag() {
        assert_eq!(round_trip(&CustomClickPayload(None)), [1, 0]);
    }

    #[test]
    fn compound_payload() {
        let payload = CustomClickPayload::from(NbtTag::Compound(NbtCompound::from_values(vec![(
            "field".into(),
            NbtTag::String("hi".into()),
        )])));
        let bytes = round_trip(&payload);
        // the length covers everything after it: a compound with one string tag
        // (type, name length, "field", string length, "hi") and the end tag
        let tag_len = 1 + (1 + 2 + 5 + 2 + 2) + 1;
        assert_eq!(bytes[0] as usize, tag_len);
        assert_eq!(bytes.len(), 1 + tag_len);
    }

    #[test]
    fn any_tag_can_be_a_payload() {
        // vanilla's payload is an Optional<Tag>, not only a compound
        let bytes = round_trip(&CustomClickPayload::from(NbtTag::Int(7)));
        assert_eq!(bytes, [5, 3, 0, 0, 0, 7]);
    }
}
