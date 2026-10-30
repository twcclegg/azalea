use azalea_block::BlockState;
use azalea_buf::AzBuf;
use azalea_core::{color::RgbColor, entity_id::MinecraftEntityId, position::BlockPos};
use azalea_inventory::ItemStack;
use azalea_registry::builtin::ParticleKind;

// the order of this enum must be kept in sync with ParticleKind, otherwise
// we get errors parsing particles.

/// A [`ParticleKind`] with data potentially attached to it.
#[cfg_attr(feature = "bevy_ecs", derive(bevy_ecs::component::Component))]
#[derive(AzBuf, Clone, Debug, PartialEq)]
pub enum Particle {
    AngryVillager,
    Block(BlockParticle),
    BlockMarker(BlockParticle),
    Bubble,
    SulfurBubbles,
    NoxiousGas,
    NoxiousGasCloud,
    Geyser(GeyserParticle),
    GeyserBase(GeyserBaseParticle),
    GeyserPoof(GeyserBaseParticle),
    GeyserPlume(GeyserParticle),
    Cloud,
    CopperFireFlame,
    Crit,
    DamageIndicator,
    DragonBreath,
    DrippingLava,
    FallingLava,
    LandingLava,
    DrippingWater,
    FallingWater,
    Dust(DustParticle),
    DustColorTransition(DustColorTransitionParticle),
    Effect,
    ElderGuardian,
    EnchantedHit,
    Enchant,
    EndRod,
    EntityEffect(ColorParticle),
    ExplosionEmitter,
    Explosion,
    Gust,
    SmallGust,
    GustEmitterLarge,
    GustEmitterSmall,
    SonicBoom,
    FallingDust(BlockParticle),
    Firework,
    Fishing,
    Flame,
    Infested,
    CherryLeaves,
    PaleOakLeaves,
    TintedLeaves,
    SculkSoul,
    SculkCharge(SculkChargeParticle),
    SculkChargePop,
    SoulFireFlame,
    Soul,
    Flash,
    HappyVillager,
    Composter,
    Heart,
    InstantEffect,
    Item(ItemParticle),
    Vibration(Box<VibrationParticle>),
    Trail(Box<TrailParticle>),
    PauseMobGrowth,
    ResetMobGrowth,
    ItemSlime,
    ItemCobweb,
    ItemSnowball,
    LargeSmoke,
    Lava,
    Mycelium,
    Note,
    Poof,
    Portal,
    Rain,
    Smoke,
    WhiteSmoke,
    Sneeze,
    Spit,
    SquidInk,
    SweepAttack,
    TotemOfUndying,
    Underwater,
    Splash,
    Witch,
    BubblePop,
    CurrentDown,
    BubbleColumnUp,
    Nautilus,
    Dolphin,
    CampfireCosySmoke,
    CampfireSignalSmoke,
    DrippingHoney,
    FallingHoney,
    LandingHoney,
    FallingNectar,
    FallingSporeBlossom,
    Ash,
    CrimsonSpore,
    WarpedSpore,
    SporeBlossomAir,
    DrippingObsidianTear,
    FallingObsidianTear,
    LandingObsidianTear,
    ReversePortal,
    WhiteAsh,
    SmallFlame,
    Snowflake,
    DrippingDripstoneLava,
    FallingDripstoneLava,
    DrippingDripstoneWater,
    FallingDripstoneWater,
    GlowSquidInk,
    Glow,
    WaxOn,
    WaxOff,
    ElectricSpark,
    Scrape,
    Shriek(ShriekParticle),
    EggCrack,
    DustPlume,
    TrialSpawnerDetection,
    TrialSpawnerDetectionOminous,
    VaultConnection,
    DustPillar,
    OminousSpawning,
    RaidOmen,
    TrialOmen,
    BlockCrumble,
    Firefly,
    SulfurCubeGoo,
}

impl From<ParticleKind> for Particle {
    /// Convert a particle kind into particle data.
    ///
    /// If the particle has data attached (like block particles), then it's set
    /// to the default.
    fn from(kind: ParticleKind) -> Self {
        // this is mostly just here so it fails to compile when a new particle
        // is added to ParticleKind, since `Particle` has to be updated manually
        match kind {
            ParticleKind::AngryVillager => Self::AngryVillager,
            ParticleKind::Block => Self::Block(BlockParticle::default()),
            ParticleKind::BlockMarker => Self::BlockMarker(BlockParticle::default()),
            ParticleKind::Bubble => Self::Bubble,
            ParticleKind::Cloud => Self::Cloud,
            ParticleKind::Crit => Self::Crit,
            ParticleKind::DamageIndicator => Self::DamageIndicator,
            ParticleKind::DragonBreath => Self::DragonBreath,
            ParticleKind::DrippingLava => Self::DrippingLava,
            ParticleKind::FallingLava => Self::FallingLava,
            ParticleKind::LandingLava => Self::LandingLava,
            ParticleKind::DrippingWater => Self::DrippingWater,
            ParticleKind::FallingWater => Self::FallingWater,
            ParticleKind::Dust => Self::Dust(DustParticle::default()),
            ParticleKind::DustColorTransition => {
                Self::DustColorTransition(DustColorTransitionParticle::default())
            }
            ParticleKind::Effect => Self::Effect,
            ParticleKind::ElderGuardian => Self::ElderGuardian,
            ParticleKind::EnchantedHit => Self::EnchantedHit,
            ParticleKind::Enchant => Self::Enchant,
            ParticleKind::EndRod => Self::EndRod,
            ParticleKind::EntityEffect => Self::EntityEffect(ColorParticle::default()),
            ParticleKind::ExplosionEmitter => Self::ExplosionEmitter,
            ParticleKind::Explosion => Self::Explosion,
            ParticleKind::Gust => Self::Gust,
            ParticleKind::SonicBoom => Self::SonicBoom,
            ParticleKind::FallingDust => Self::FallingDust(BlockParticle::default()),
            ParticleKind::Firework => Self::Firework,
            ParticleKind::Fishing => Self::Fishing,
            ParticleKind::Flame => Self::Flame,
            ParticleKind::CherryLeaves => Self::CherryLeaves,
            ParticleKind::PaleOakLeaves => Self::PaleOakLeaves,
            ParticleKind::TintedLeaves => Self::TintedLeaves,
            ParticleKind::SculkSoul => Self::SculkSoul,
            ParticleKind::SculkCharge => Self::SculkCharge(SculkChargeParticle::default()),
            ParticleKind::SculkChargePop => Self::SculkChargePop,
            ParticleKind::SoulFireFlame => Self::SoulFireFlame,
            ParticleKind::Soul => Self::Soul,
            ParticleKind::Flash => Self::Flash,
            ParticleKind::HappyVillager => Self::HappyVillager,
            ParticleKind::Composter => Self::Composter,
            ParticleKind::Heart => Self::Heart,
            ParticleKind::InstantEffect => Self::InstantEffect,
            ParticleKind::Item => Self::Item(ItemParticle::default()),
            ParticleKind::Vibration => Self::Vibration(Default::default()),
            ParticleKind::ItemSlime => Self::ItemSlime,
            ParticleKind::ItemSnowball => Self::ItemSnowball,
            ParticleKind::LargeSmoke => Self::LargeSmoke,
            ParticleKind::Lava => Self::Lava,
            ParticleKind::Mycelium => Self::Mycelium,
            ParticleKind::Note => Self::Note,
            ParticleKind::Poof => Self::Poof,
            ParticleKind::Portal => Self::Portal,
            ParticleKind::Rain => Self::Rain,
            ParticleKind::Smoke => Self::Smoke,
            ParticleKind::WhiteSmoke => Self::WhiteSmoke,
            ParticleKind::Sneeze => Self::Sneeze,
            ParticleKind::Spit => Self::Spit,
            ParticleKind::SquidInk => Self::SquidInk,
            ParticleKind::SweepAttack => Self::SweepAttack,
            ParticleKind::TotemOfUndying => Self::TotemOfUndying,
            ParticleKind::Underwater => Self::Underwater,
            ParticleKind::Splash => Self::Splash,
            ParticleKind::Witch => Self::Witch,
            ParticleKind::BubblePop => Self::BubblePop,
            ParticleKind::CurrentDown => Self::CurrentDown,
            ParticleKind::BubbleColumnUp => Self::BubbleColumnUp,
            ParticleKind::Nautilus => Self::Nautilus,
            ParticleKind::Dolphin => Self::Dolphin,
            ParticleKind::CampfireCosySmoke => Self::CampfireCosySmoke,
            ParticleKind::CampfireSignalSmoke => Self::CampfireSignalSmoke,
            ParticleKind::DrippingHoney => Self::DrippingHoney,
            ParticleKind::FallingHoney => Self::FallingHoney,
            ParticleKind::LandingHoney => Self::LandingHoney,
            ParticleKind::FallingNectar => Self::FallingNectar,
            ParticleKind::FallingSporeBlossom => Self::FallingSporeBlossom,
            ParticleKind::Ash => Self::Ash,
            ParticleKind::CrimsonSpore => Self::CrimsonSpore,
            ParticleKind::WarpedSpore => Self::WarpedSpore,
            ParticleKind::SporeBlossomAir => Self::SporeBlossomAir,
            ParticleKind::DrippingObsidianTear => Self::DrippingObsidianTear,
            ParticleKind::FallingObsidianTear => Self::FallingObsidianTear,
            ParticleKind::LandingObsidianTear => Self::LandingObsidianTear,
            ParticleKind::ReversePortal => Self::ReversePortal,
            ParticleKind::WhiteAsh => Self::WhiteAsh,
            ParticleKind::SmallFlame => Self::SmallFlame,
            ParticleKind::Snowflake => Self::Snowflake,
            ParticleKind::DrippingDripstoneLava => Self::DrippingDripstoneLava,
            ParticleKind::FallingDripstoneLava => Self::FallingDripstoneLava,
            ParticleKind::DrippingDripstoneWater => Self::DrippingDripstoneWater,
            ParticleKind::FallingDripstoneWater => Self::FallingDripstoneWater,
            ParticleKind::GlowSquidInk => Self::GlowSquidInk,
            ParticleKind::Glow => Self::Glow,
            ParticleKind::WaxOn => Self::WaxOn,
            ParticleKind::WaxOff => Self::WaxOff,
            ParticleKind::ElectricSpark => Self::ElectricSpark,
            ParticleKind::Scrape => Self::Scrape,
            ParticleKind::Shriek => Self::Shriek(ShriekParticle::default()),
            ParticleKind::EggCrack => Self::EggCrack,
            ParticleKind::DustPlume => Self::DustPlume,
            ParticleKind::SmallGust => Self::SmallGust,
            ParticleKind::GustEmitterLarge => Self::GustEmitterLarge,
            ParticleKind::GustEmitterSmall => Self::GustEmitterSmall,
            ParticleKind::Infested => Self::Infested,
            ParticleKind::ItemCobweb => Self::ItemCobweb,
            ParticleKind::TrialSpawnerDetection => Self::TrialSpawnerDetection,
            ParticleKind::TrialSpawnerDetectionOminous => Self::TrialSpawnerDetectionOminous,
            ParticleKind::VaultConnection => Self::VaultConnection,
            ParticleKind::DustPillar => Self::DustPillar,
            ParticleKind::OminousSpawning => Self::OminousSpawning,
            ParticleKind::RaidOmen => Self::RaidOmen,
            ParticleKind::TrialOmen => Self::TrialOmen,
            ParticleKind::Trail => Self::Trail(Box::default()),
            ParticleKind::BlockCrumble => Self::BlockCrumble,
            ParticleKind::Firefly => Self::Firefly,
            ParticleKind::CopperFireFlame => Self::CopperFireFlame,
            ParticleKind::PauseMobGrowth => Self::PauseMobGrowth,
            ParticleKind::ResetMobGrowth => Self::ResetMobGrowth,
            ParticleKind::SulfurBubbles => Self::SulfurBubbles,
            ParticleKind::NoxiousGas => Self::NoxiousGas,
            ParticleKind::NoxiousGasCloud => Self::NoxiousGasCloud,
            ParticleKind::Geyser => Self::Geyser(GeyserParticle::default()),
            ParticleKind::GeyserBase => Self::GeyserBase(GeyserBaseParticle::default()),
            ParticleKind::GeyserPoof => Self::GeyserPoof(GeyserBaseParticle::default()),
            ParticleKind::GeyserPlume => Self::GeyserPlume(GeyserParticle::default()),
            ParticleKind::SulfurCubeGoo => Self::SulfurCubeGoo,
        }
    }
}

impl Default for Particle {
    fn default() -> Self {
        Self::EntityEffect(ColorParticle::default())
    }
}

#[derive(AzBuf, Clone, Debug, Default, PartialEq)]
pub struct BlockParticle {
    pub block_state: BlockState,
}
#[derive(AzBuf, Clone, Debug, Default, PartialEq)]
pub struct DustParticle {
    pub color: RgbColor,
    /// The scale, will be clamped between 0.01 and 4.
    pub scale: f32,
}

#[derive(AzBuf, Clone, Debug, Default, PartialEq)]
pub struct DustColorTransitionParticle {
    pub from: RgbColor,
    pub to: RgbColor,
    /// The scale, will be clamped between 0.01 and 4.
    pub scale: f32,
}

#[derive(AzBuf, Clone, Debug, Default, PartialEq)]
pub struct ColorParticle {
    pub color: RgbColor,
}

#[derive(AzBuf, Clone, Debug, Default, PartialEq)]
pub struct ItemParticle {
    pub item: ItemStack,
}

#[derive(AzBuf, Clone, Debug, Default, PartialEq)]
pub struct VibrationParticle {
    pub position: PositionSource,
    #[var]
    pub ticks: u32,
}

#[derive(AzBuf, Clone, Debug, PartialEq)]
pub enum PositionSource {
    Block(BlockPos),
    Entity {
        #[var]
        id: MinecraftEntityId,
        y_offset: f32,
    },
}
impl Default for PositionSource {
    fn default() -> Self {
        // bad default but hopefully it never gets used anyways
        Self::Block(BlockPos::default())
    }
}

/// A trail particle's options, as vanilla's `TrailParticleOption`. The color is
/// ARGB.
#[derive(AzBuf, Clone, Debug, Default, PartialEq)]
pub struct TrailParticle {
    pub target: azalea_core::position::Vec3,
    pub color: u32,
    #[var]
    pub duration: i32,
}

#[derive(AzBuf, Clone, Debug, Default, PartialEq)]
pub struct SculkChargeParticle {
    pub roll: f32,
}

#[derive(AzBuf, Clone, Debug, Default, PartialEq)]
pub struct ShriekParticle {
    /// The time in ticks before the particle is displayed.
    #[var]
    pub delay: i32,
}

#[derive(AzBuf, Clone, Debug, Default, PartialEq)]
pub struct GeyserParticle {
    pub water_blocks: i32,
}

#[derive(AzBuf, Clone, Debug, Default, PartialEq)]
pub struct GeyserBaseParticle {
    pub water_blocks: i32,
    pub burst_impulse_base: f32,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use azalea_buf::{AzBuf, AzBufVar};
    use azalea_registry::Registry;

    use super::*;

    #[test]
    fn trails_carry_their_options() {
        // As 26.2 writes a trail: its type id, the target (three doubles), the
        // colour (int), the duration (VarInt), then whatever follows in the
        // packet.
        let mut b = Vec::new();
        ParticleKind::Trail
            .to_u32()
            .azalea_write_var(&mut b)
            .unwrap();
        for v in [10.5f64, 70., -4.25] {
            b.extend(v.to_be_bytes());
        }
        b.extend(0xFF33_AA55u32.to_be_bytes());
        40i32.azalea_write_var(&mut b).unwrap();
        b.push(0xEE);
        let mut cursor = Cursor::new(b.as_slice());
        let particle = Particle::azalea_read(&mut cursor).unwrap();
        let Particle::Trail(trail) = particle else {
            panic!("{particle:?}")
        };
        assert_eq!(
            (trail.target.x, trail.target.y, trail.target.z),
            (10.5, 70., -4.25)
        );
        assert_eq!((trail.color, trail.duration), (0xFF33_AA55, 40));
        assert_eq!(
            cursor.position() as usize,
            b.len() - 1,
            "stops where the trail ends"
        );
    }
}
