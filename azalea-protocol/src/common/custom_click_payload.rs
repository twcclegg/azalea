use std::io::{self, Cursor, Read, Write};

use azalea_buf::{AzBuf, AzBufVar, BufReadError, BufReadErrorRepr};
use simdnbt::owned::Nbt;

/// The NBT payload of a custom click action, as vanilla's
/// `ServerboundCustomClickActionPacket.UNTRUSTED_TAG_CODEC` writes it.
///
/// That's `optionalTagCodec(...).apply(lengthPrefixed(65536))`: the tag as
/// unnamed network NBT (or a lone end tag when there's no payload), with its
/// length in bytes as a VarInt before it. So an empty payload is written as
/// `[1, 0]`, not `[0]`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CustomClickPayload(pub Nbt);

/// The most bytes vanilla accepts for a payload.
pub const MAX_CUSTOM_CLICK_PAYLOAD_LENGTH: u32 = 65536;

impl From<Nbt> for CustomClickPayload {
    fn from(nbt: Nbt) -> Self {
        Self(nbt)
    }
}

impl AzBuf for CustomClickPayload {
    fn azalea_read(buf: &mut Cursor<&[u8]>) -> Result<Self, BufReadError> {
        let length = u32::azalea_read_var(buf)?;
        if length > MAX_CUSTOM_CLICK_PAYLOAD_LENGTH {
            return Err(BufReadErrorRepr::VecLengthTooLong {
                length,
                max_length: MAX_CUSTOM_CLICK_PAYLOAD_LENGTH,
            }
            .into());
        }
        let mut payload = vec![0; length as usize];
        buf.read_exact(&mut payload)?;
        if payload.is_empty() {
            return Ok(Self(Nbt::None));
        }
        Ok(Self(Nbt::azalea_read(&mut Cursor::new(
            payload.as_slice(),
        ))?))
    }

    fn azalea_write(&self, buf: &mut impl Write) -> io::Result<()> {
        let mut payload = Vec::new();
        self.0.azalea_write(&mut payload)?;
        (payload.len() as u32).azalea_write_var(buf)?;
        buf.write_all(&payload)
    }
}

#[cfg(test)]
mod tests {
    use simdnbt::owned::{BaseNbt, NbtCompound, NbtTag};

    use super::*;

    #[test]
    fn empty_payload_is_a_length_prefixed_end_tag() {
        let mut bytes = Vec::new();
        CustomClickPayload(Nbt::None)
            .azalea_write(&mut bytes)
            .unwrap();
        assert_eq!(bytes, [1, 0]);
        let read = CustomClickPayload::azalea_read(&mut Cursor::new(bytes.as_slice())).unwrap();
        assert_eq!(read, CustomClickPayload(Nbt::None));
    }

    #[test]
    fn payload_round_trips() {
        let payload = CustomClickPayload(Nbt::Some(BaseNbt::new(
            "",
            NbtCompound::from_values(vec![("field".into(), NbtTag::String("hi".into()))]),
        )));
        let mut bytes = Vec::new();
        payload.azalea_write(&mut bytes).unwrap();
        // the length covers everything after it: a compound with one string tag
        // (type, name length, "field", string length, "hi") and the end tag
        let tag_len = 1 + (1 + 2 + 5 + 2 + 2) + 1;
        assert_eq!(bytes[0] as usize, tag_len);
        assert_eq!(bytes.len(), 1 + tag_len);

        let mut cursor = Cursor::new(bytes.as_slice());
        assert_eq!(
            CustomClickPayload::azalea_read(&mut cursor).unwrap(),
            payload
        );
        assert_eq!(cursor.position() as usize, bytes.len());
    }
}
