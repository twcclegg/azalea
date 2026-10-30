use azalea_client::{client_chat::SendChatEvent, test_utils::prelude::*};
use azalea_protocol::packets::{
    ConnectionProtocol,
    game::{
        ClientboundCommands, ServerboundGamePacket,
        c_commands::{BrigadierNodeStub, BrigadierParser, EntityParser, NodeType},
    },
};

fn node(node_type: NodeType, children: Vec<u32>) -> BrigadierNodeStub {
    BrigadierNodeStub {
        is_executable: children.is_empty(),
        children,
        redirect_node: None,
        node_type,
        is_restricted: false,
    }
}

#[test]
fn test_send_signed_command() {
    let _lock = init();

    let mut simulation = Simulation::new(ConnectionProtocol::Game);
    let sent_packets = SentPackets::new(&mut simulation);
    simulation.receive_packet(default_login_packet());
    // msg <targets> <message>, and time set day
    simulation.receive_packet(ClientboundCommands {
        entries: vec![
            node(NodeType::Root, vec![1, 4]),
            node(NodeType::Literal { name: "msg".into() }, vec![2]),
            node(
                NodeType::Argument {
                    name: "targets".into(),
                    parser: BrigadierParser::Entity(EntityParser {
                        single: false,
                        players_only: true,
                    }),
                    suggestions_type: None,
                },
                vec![3],
            ),
            node(
                NodeType::Argument {
                    name: "message".into(),
                    parser: BrigadierParser::Message,
                    suggestions_type: None,
                },
                vec![],
            ),
            node(
                NodeType::Literal {
                    name: "time".into(),
                },
                vec![5],
            ),
            node(NodeType::Literal { name: "set".into() }, vec![6]),
            node(NodeType::Literal { name: "day".into() }, vec![]),
        ],
        root_index: 0,
    });
    simulation.tick();
    sent_packets.clear();

    for content in ["/msg Kid hello there", "/time set day"] {
        simulation.write_message(SendChatEvent {
            entity: simulation.entity,
            content: content.to_owned(),
        });
    }
    simulation.tick();

    let commands = sent_packets
        .list
        .lock()
        .iter()
        .filter_map(|p| match p {
            ServerboundGamePacket::ChatCommandSigned(p) => Some(format!(
                "signed: {} ({} signatures)",
                p.command,
                p.argument_signatures.len()
            )),
            ServerboundGamePacket::ChatCommand(p) => Some(format!("unsigned: {}", p.command)),
            _ => None,
        })
        .collect::<Vec<_>>();
    // the simulation has no chat session, so there's nothing to sign with
    assert_eq!(
        commands,
        [
            "signed: msg Kid hello there (0 signatures)",
            "unsigned: time set day"
        ]
    );
}
