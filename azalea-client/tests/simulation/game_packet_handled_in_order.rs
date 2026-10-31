use std::sync::Arc;

use azalea_client::{packet::game::GamePacketHandledEvent, test_utils::prelude::*};
use azalea_core::{entity_id::MinecraftEntityId, position::ChunkPos};
use azalea_entity::indexing::EntityIdIndex;
use azalea_protocol::packets::{
    ConnectionProtocol,
    game::{ClientboundGamePacket, ClientboundRemoveEntities},
};
use azalea_registry::builtin::EntityKind;
use bevy_ecs::prelude::*;
use parking_lot::Mutex;

#[test]
fn test_game_packet_handled_in_order() {
    let _lock = init();

    let mut simulation = Simulation::new(ConnectionProtocol::Game);
    simulation.receive_packet(default_login_packet());
    simulation.receive_packet(make_basic_empty_chunk(ChunkPos::new(0, 0), (384 + 64) / 16));
    simulation.tick();

    // for each packet: whether it removes entities, and whether entity 123 was known
    // to the client right after it was handled
    let seen = Arc::new(Mutex::new(Vec::new()));
    let seen_in_observer = seen.clone();
    simulation.app.add_observer(
        move |event: On<GamePacketHandledEvent>, query: Query<&EntityIdIndex>| {
            let known = query
                .get(event.entity)
                .unwrap()
                .get_by_minecraft_entity(MinecraftEntityId(123))
                .is_some();
            let removes = matches!(
                event.packet.as_ref(),
                ClientboundGamePacket::RemoveEntities(_)
            );
            seen_in_observer.lock().push((removes, known));
        },
    );

    // both arrive in the same batch, so by the time ReceiveGamePacketEvent is read,
    // the entity is already gone
    simulation.receive_packet(make_basic_add_entity(EntityKind::Cow, 123, (0.5, 64., 0.5)));
    simulation.receive_packet(ClientboundRemoveEntities {
        entity_ids: vec![MinecraftEntityId(123)],
    });
    simulation.tick();

    assert_eq!(*seen.lock(), [(false, true), (true, false)]);
}
