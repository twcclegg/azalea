use azalea_buf::AzBuf;
use azalea_core::{position::Vec3, sound::CustomSound};
use azalea_entity::particle::Particle;
use azalea_protocol_macros::ClientboundGamePacket;
use azalea_registry::{Holder, builtin::SoundEvent};

#[derive(AzBuf, ClientboundGamePacket, Clone, Debug, PartialEq)]
pub struct ClientboundExplode {
    pub center: Vec3,
    pub radius: f32,
    pub block_count: i32,
    pub player_knockback: Option<Vec3>,
    pub explosion_particle: Particle,
    pub explosion_sound: Holder<SoundEvent, CustomSound>,
    pub block_particles: Vec<Weighted<ExplosionParticleInfo>>,
}

#[derive(AzBuf, Clone, Debug, PartialEq)]
pub struct Weighted<T: AzBuf> {
    pub value: T,
    #[var]
    pub weight: i32,
}

#[derive(AzBuf, Clone, Debug, PartialEq)]
pub struct ExplosionParticleInfo {
    pub particle: Particle,
    pub scaling: f32,
    pub speed: f32,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use azalea_buf::{AzBuf, AzBufVar};
    use azalea_registry::{Holder, Registry, builtin::{ParticleKind, SoundEvent}};

    use super::*;

    /// The packet as 26.2 writes it, with the sound given as `sound`.
    fn bytes(sound: &[u8]) -> Vec<u8> {
        let mut b = Vec::new();
        for v in [1.5f64, 64., -3.5] {
            b.extend(v.to_be_bytes());
        }
        b.extend(4f32.to_be_bytes());
        b.extend(12i32.to_be_bytes());
        b.push(0); // no knockback
        ParticleKind::Explosion.to_u32().azalea_write_var(&mut b).unwrap();
        b.extend(sound);
        b.push(0); // no block particles
        b
    }

    #[test]
    fn explosion_sounds_are_holders() {
        let mut sound = Vec::new();
        (SoundEvent::EntityGenericExplode.to_u32() + 1).azalea_write_var(&mut sound).unwrap();
        let packet = ClientboundExplode::azalea_read(&mut Cursor::new(bytes(&sound).as_slice())).unwrap();
        assert_eq!(packet.explosion_sound, Holder::Reference(SoundEvent::EntityGenericExplode));
        assert_eq!((packet.radius, packet.block_count), (4., 12));

        // A sound sent inline: 0, its id, and an optional fixed range.
        let mut sound = vec![0, 13];
        sound.extend(b"custom:kaboom");
        sound.push(1);
        sound.extend(32f32.to_be_bytes());
        let packet = ClientboundExplode::azalea_read(&mut Cursor::new(bytes(&sound).as_slice())).unwrap();
        let Holder::Direct(custom) = packet.explosion_sound else { panic!("inline") };
        assert_eq!((custom.sound_id.to_string().as_str(), custom.range), ("custom:kaboom", Some(32.)));
        assert!(packet.block_particles.is_empty());
    }
}
