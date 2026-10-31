use azalea_chat::FormattedText;
use azalea_client::{
    client_chat::{
        ChatKind as SendKind, handler::SendChatKindEvent, last_seen::last_seen_checksum,
    },
    test_utils::prelude::*,
};
use azalea_crypto::signing::MessageSignature;
use azalea_protocol::packets::{
    ConnectionProtocol,
    game::{
        ClientboundPlayerChat, ServerboundGamePacket,
        c_player_chat::{
            ChatTypeBound, FilterMask, PackedLastSeenMessages, PackedSignedMessageBody,
        },
    },
};
use azalea_registry::{DataRegistry, Holder, data::ChatKind};
use uuid::Uuid;

fn player_chat(signature: Option<MessageSignature>) -> ClientboundPlayerChat {
    ClientboundPlayerChat {
        global_index: 0,
        sender: Uuid::nil(),
        index: 0,
        signature,
        body: PackedSignedMessageBody {
            content: "hi".to_owned(),
            timestamp: 0,
            salt: 0,
            last_seen: PackedLastSeenMessages { entries: vec![] },
        },
        unsigned_content: None,
        filter_mask: FilterMask::PassThrough,
        chat_type: ChatTypeBound {
            chat_type: Holder::Reference(ChatKind::new_raw(0)),
            name: FormattedText::from("someone".to_owned()),
            target_name: None,
        },
    }
}

#[test]
fn test_acknowledge_seen_chat() {
    let _lock = init();

    let mut simulation = Simulation::new(ConnectionProtocol::Game);
    let sent_packets = SentPackets::new(&mut simulation);
    simulation.receive_packet(default_login_packet());
    simulation.tick();

    let first = MessageSignature { bytes: [1; 256] };
    let second = MessageSignature { bytes: [2; 256] };
    simulation.receive_packet(player_chat(Some(first.clone())));
    simulation.receive_packet(player_chat(None));
    simulation.receive_packet(player_chat(Some(second.clone())));
    simulation.tick();
    sent_packets.clear();

    simulation.write_message(SendChatKindEvent {
        entity: simulation.entity,
        content: "hello".to_owned(),
        kind: SendKind::Message,
    });
    simulation.tick();

    let chat = sent_packets
        .list
        .lock()
        .iter()
        .find_map(|p| match p {
            ServerboundGamePacket::Chat(chat) => Some(chat.clone()),
            _ => None,
        })
        .expect("a chat packet should be sent");
    let update = chat.last_seen_messages;
    assert_eq!(update.offset, 2);
    // the window has 20 slots and the newest messages are at the end
    assert!((0..18).all(|i| !update.acknowledged.index(i)));
    assert!(update.acknowledged.index(18) && update.acknowledged.index(19));
    assert_eq!(update.checksum, last_seen_checksum(&[first, second]));
}

#[test]
fn test_acknowledge_many_pending_messages() {
    let _lock = init();

    let mut simulation = Simulation::new(ConnectionProtocol::Game);
    let sent_packets = SentPackets::new(&mut simulation);
    simulation.receive_packet(default_login_packet());
    simulation.tick();
    sent_packets.clear();

    for n in 0..65u8 {
        simulation.receive_packet(player_chat(Some(MessageSignature { bytes: [n; 256] })));
    }
    simulation.tick();

    let acks = sent_packets
        .list
        .lock()
        .iter()
        .filter_map(|p| match p {
            ServerboundGamePacket::ChatAck(ack) => Some(ack.messages),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(acks, [65]);
}
