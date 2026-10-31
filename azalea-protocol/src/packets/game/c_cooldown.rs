use azalea_buf::AzBuf;
use azalea_registry::identifier::Identifier;
use azalea_protocol_macros::ClientboundGamePacket;

/// An item cooldown starting (or ending, with a duration of 0) for a cooldown
/// group: an item's id, or the group named by its `use_cooldown` component.
#[derive(AzBuf, ClientboundGamePacket, Clone, Debug, PartialEq)]
pub struct ClientboundCooldown {
    pub cooldown_group: Identifier,
    #[var]
    pub duration: u32,
}
