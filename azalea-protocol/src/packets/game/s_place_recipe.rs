use azalea_buf::AzBuf;
use azalea_protocol_macros::ServerboundGamePacket;

#[derive(AzBuf, Clone, Debug, PartialEq, ServerboundGamePacket)]
pub struct ServerboundPlaceRecipe {
    #[var]
    pub container_id: i32,
    /// The recipe's display id, as given by `ClientboundRecipeBookAdd`.
    #[var]
    pub recipe: u32,
    pub shift_down: bool,
}

#[cfg(test)]
mod tests {
    use azalea_buf::AzBuf;

    use super::*;

    #[test]
    fn written_as_the_game_reads_it() {
        // ServerboundPlaceRecipePacket: container id (VarInt), RecipeDisplayId (VarInt), whether
        // to place as many as fit.
        let mut bytes = Vec::new();
        ServerboundPlaceRecipe { container_id: 3, recipe: 300, shift_down: true }.azalea_write(&mut bytes).unwrap();
        assert_eq!(bytes, [3, 0xAC, 0x02, 1]);
        let mut bytes = Vec::new();
        crate::packets::game::ServerboundRecipeBookSeenRecipe { recipe: 5 }.azalea_write(&mut bytes).unwrap();
        assert_eq!(bytes, [5]);
    }
}
