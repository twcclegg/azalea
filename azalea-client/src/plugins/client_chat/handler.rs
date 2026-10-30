use std::time::{SystemTime, UNIX_EPOCH};

use azalea_protocol::packets::{
    Packet,
    game::{ServerboundChat, ServerboundChatCommand},
};
use bevy_ecs::prelude::*;

use super::{ChatKind, last_seen::LastSeenMessagesTracker};
use crate::packet::game::SendGamePacketEvent;
#[cfg(feature = "online-mode")]
use crate::{account::Account, chat_signing::ChatSigningSession};

/// Send a chat packet to the server of a specific kind (chat message or
/// command). Usually you just want [`SendChatEvent`] instead.
///
/// Usually setting the kind to `Message` will make it send a chat message even
/// if it starts with a slash, but some server implementations will always do a
/// command if it starts with a slash.
///
/// If you're wondering why this isn't two separate events, it's so ordering is
/// preserved if multiple chat messages and commands are sent at the same time.
///
/// [`SendChatEvent`]: super::SendChatEvent
#[derive(Message)]
pub struct SendChatKindEvent {
    pub entity: Entity,
    pub content: String,
    pub kind: ChatKind,
}

pub fn handle_send_chat_kind_event(
    mut events: MessageReader<SendChatKindEvent>,
    mut commands: Commands,
    mut last_seen_query: Query<&mut LastSeenMessagesTracker>,
    #[cfg(feature = "online-mode")] mut query: Query<(&Account, &mut ChatSigningSession)>,
) {
    for event in events.read() {
        let content = event
            .content
            .chars()
            .filter(|c| !matches!(c, '\x00'..='\x1F' | '\x7F' | '§'))
            .take(256)
            .collect::<String>();

        let timestamp = SystemTime::now();

        let packet = match event.kind {
            ChatKind::Message => {
                let salt = azalea_crypto::signing::make_salt();
                // like vanilla, every chat message acknowledges the messages
                // we've seen, whether or not it's signed
                let (last_seen, last_seen_messages) = last_seen_query
                    .get_mut(event.entity)
                    .map(|mut tracker| tracker.generate_and_apply_update())
                    .unwrap_or_default();

                #[cfg(feature = "online-mode")]
                let signature = if let Ok((account, mut chat_session)) = query.get_mut(event.entity)
                {
                    Some(create_signature(
                        account,
                        &mut chat_session,
                        salt,
                        timestamp,
                        &content,
                        last_seen,
                    ))
                } else {
                    None
                };
                #[cfg(not(feature = "online-mode"))]
                let signature = {
                    let _ = last_seen;
                    None
                };

                ServerboundChat {
                    message: content,
                    timestamp: timestamp
                        .duration_since(UNIX_EPOCH)
                        .expect("Time shouldn't be before epoch")
                        .as_millis()
                        .try_into()
                        .expect("Instant should fit into a u64"),
                    salt,
                    signature,
                    last_seen_messages,
                }
            }
            .into_variant(),
            ChatKind::Command => {
                // TODO: commands that require chat signing
                ServerboundChatCommand { command: content }.into_variant()
            }
        };

        commands.trigger(SendGamePacketEvent::new(event.entity, packet));
    }
}

#[cfg(feature = "online-mode")]
pub fn create_signature(
    account: &Account,
    chat_session: &mut ChatSigningSession,
    salt: u64,
    timestamp: SystemTime,
    message: &str,
    last_seen_messages: Vec<azalea_crypto::signing::MessageSignature>,
) -> azalea_crypto::signing::MessageSignature {
    use azalea_crypto::signing::SignChatMessageOptions;

    let certs = account
        .certs()
        .expect("certs shouldn't be set back to None");

    let signature = azalea_crypto::signing::sign_chat_message(&SignChatMessageOptions {
        account_uuid: account.uuid(),
        chat_session_uuid: chat_session.session_id,
        message_index: chat_session.messages_sent,
        salt,
        timestamp,
        message: message.to_owned(),
        last_seen_messages,
        private_key: certs.private_key.clone(),
    });

    chat_session.messages_sent += 1;

    signature
}
