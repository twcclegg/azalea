use azalea_buf::AzBuf;
use azalea_registry::identifier::Identifier;
use azalea_protocol_macros::ServerboundGamePacket;

/// A dialog button's custom action, or a custom click event.
///
/// The payload is length-prefixed unnamed NBT: the bytes of a tag, or a lone 0
/// (an end tag) for none.
#[derive(AzBuf, Clone, Debug, PartialEq, ServerboundGamePacket)]
pub struct ServerboundCustomClickAction {
    pub id: Identifier,
    pub payload: Vec<u8>,
}
