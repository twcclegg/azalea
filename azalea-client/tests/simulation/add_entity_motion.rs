use azalea_client::test_utils::prelude::*;
use azalea_core::{
    delta::LpVec3,
    entity_id::MinecraftEntityId,
    position::{ChunkPos, Vec3},
};
use azalea_entity::{LookDirection, Physics, indexing::EntityIdIndex};
use azalea_protocol::packets::ConnectionProtocol;
use azalea_registry::builtin::EntityKind;

#[test]
fn test_add_entity_motion() {
    let _lock = init();

    let mut simulation = Simulation::new(ConnectionProtocol::Game);
    simulation.receive_packet(default_login_packet());
    simulation.receive_packet(make_basic_empty_chunk(ChunkPos::new(0, 0), (384 + 64) / 16));
    simulation.tick();

    // an arrow, shot facing south-west and slightly up
    let mut packet = make_basic_add_entity(EntityKind::Arrow, 123, (0.5, 70., 0.5));
    let velocity = Vec3::new(-1., 0.5, 2.);
    packet.movement = LpVec3::from_vec3(velocity);
    packet.y_rot = 32;
    packet.x_rot = -16;
    simulation.receive_packet(packet);
    simulation.tick();

    let entity = simulation.query_self::<&EntityIdIndex, _>(|index| {
        index
            .get_by_minecraft_entity(MinecraftEntityId(123))
            .unwrap()
    });
    let world = simulation.app.world();
    let physics = world.get::<Physics>(entity).unwrap();
    // the packet's velocity is compressed, so it's only close
    assert!(
        (physics.velocity - velocity).length() < 1e-3,
        "{:?}",
        physics.velocity
    );
    let direction = world.get::<LookDirection>(entity).unwrap();
    assert_eq!((direction.y_rot(), direction.x_rot()), (45., -22.5));
}
