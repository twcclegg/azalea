use azalea_buf::AzBuf;
use azalea_protocol_macros::ServerboundGamePacket;

#[derive(AzBuf, Clone, Debug, PartialEq, ServerboundGamePacket)]
pub struct ServerboundRecipeBookSeenRecipe {
    /// The recipe's display id, as given by `ClientboundRecipeBookAdd`.
    #[var]
    pub recipe: u32,
}
