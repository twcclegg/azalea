use azalea_client::{
    test_utils::prelude::*,
    world_border::{BorderStatus, WorldBorder},
};
use azalea_protocol::packets::{
    ConnectionProtocol,
    game::{ClientboundInitializeBorder, ClientboundSetBorderCenter, ClientboundSetBorderSize},
};

#[test]
fn test_world_border() {
    let _lock = init();

    let mut simulation = Simulation::new(ConnectionProtocol::Game);
    simulation.receive_packet(default_login_packet());
    simulation.tick();
    assert_eq!(
        simulation.component::<WorldBorder>(),
        WorldBorder::default()
    );

    // shrinking from 200 to 100 over 20 ticks
    simulation.receive_packet(ClientboundInitializeBorder {
        new_center_x: 10.,
        new_center_z: -10.,
        old_size: 200.,
        new_size: 100.,
        lerp_time: 20,
        new_absolute_max_size: 29_999_984,
        warning_blocks: 5,
        warning_time: 15,
    });
    simulation.tick();
    let border = simulation.component::<WorldBorder>();
    assert_eq!(border.status(), BorderStatus::Shrinking);
    assert_eq!(border.size(), 195.);

    for _ in 0..19 {
        simulation.tick();
    }
    let border = simulation.component::<WorldBorder>();
    assert_eq!(border.status(), BorderStatus::Stationary);
    assert_eq!((border.min_x(), border.max_z()), (-40., 40.));

    simulation.receive_packet(ClientboundSetBorderCenter {
        new_center_x: 0.,
        new_center_z: 0.,
    });
    simulation.receive_packet(ClientboundSetBorderSize { size: 10. });
    simulation.tick();
    let border = simulation.component::<WorldBorder>();
    assert_eq!((border.min_x(), border.max_x()), (-5., 5.));
}
