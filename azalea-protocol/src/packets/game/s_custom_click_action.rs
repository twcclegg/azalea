use azalea_buf::AzBuf;
use azalea_registry::identifier::Identifier;
use azalea_protocol_macros::ServerboundGamePacket;

use crate::common::custom_click_payload::CustomClickPayload;

/// A dialog button's custom action, or a custom click event.
#[derive(AzBuf, Clone, Debug, PartialEq, ServerboundGamePacket)]
pub struct ServerboundCustomClickAction {
    pub id: Identifier,
    pub payload: CustomClickPayload,
}
