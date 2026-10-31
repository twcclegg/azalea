use azalea_client::test_utils::prelude::*;
use azalea_core::attribute_modifier_operation::AttributeModifierOperation;
use azalea_entity::{Attributes, attributes::sprinting_modifier};
use azalea_inventory::components::AttributeModifier;
use azalea_protocol::packets::{
    ConnectionProtocol,
    game::{ClientboundUpdateAttributes, c_update_attributes::AttributeSnapshot},
};
use azalea_registry::{builtin::Attribute, identifier::Identifier};

fn speed_effect() -> AttributeModifier {
    AttributeModifier {
        id: Identifier::new("effect.speed"),
        amount: 0.4,
        operation: AttributeModifierOperation::AddMultipliedTotal,
    }
}

#[test]
fn test_update_attributes() {
    let _lock = init();

    let mut simulation = Simulation::new(ConnectionProtocol::Game);
    simulation.receive_packet(default_login_packet());
    simulation.tick();

    // we're sprinting, but the server doesn't know yet
    simulation.with_component_mut::<Attributes>(|attributes| {
        attributes.movement_speed.insert(sprinting_modifier());
    });

    let entity_id = simulation.minecraft_entity_id();
    simulation.receive_packet(ClientboundUpdateAttributes {
        entity_id,
        values: vec![
            AttributeSnapshot {
                attribute: Attribute::MovementSpeed,
                base: 0.1,
                modifiers: vec![speed_effect()],
            },
            AttributeSnapshot {
                attribute: Attribute::StepHeight,
                base: 1.0,
                modifiers: vec![],
            },
        ],
    });
    simulation.tick();

    let attributes = simulation.component::<Attributes>();
    let expected = 0.1 * 1.4 * (1. + sprinting_modifier().amount);
    assert!((attributes.movement_speed.calculate() - expected).abs() < 1e-9);
    assert_eq!(attributes.step_height.calculate(), 1.0);

    // the effect ends, and the server now thinks we're sprinting while we've stopped
    simulation.with_component_mut::<Attributes>(|attributes| {
        attributes.movement_speed.remove(&sprinting_modifier().id);
    });
    simulation.receive_packet(ClientboundUpdateAttributes {
        entity_id,
        values: vec![AttributeSnapshot {
            attribute: Attribute::MovementSpeed,
            base: 0.1,
            modifiers: vec![sprinting_modifier()],
        }],
    });
    simulation.tick();

    let attributes = simulation.component::<Attributes>();
    assert!((attributes.movement_speed.calculate() - 0.1).abs() < 1e-9);
}
