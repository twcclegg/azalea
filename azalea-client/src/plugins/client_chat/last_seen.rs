//! Tracking which signed chat messages we've seen, so our own signed messages
//! can acknowledge them.

use azalea_core::bitset::FixedBitSet;
use azalea_crypto::signing::MessageSignature;
use azalea_protocol::packets::game::s_chat::LastSeenMessagesUpdate;
use bevy_ecs::prelude::*;

/// How many seen messages are tracked and acknowledged at most.
pub const LAST_SEEN_MESSAGES_MAX_LENGTH: usize = 20;
/// How many messages may be pending before we acknowledge them with a
/// `ServerboundChatAck`, without waiting to send a chat message.
pub const MAX_PENDING_MESSAGES: u32 = 64;

/// The signed messages we've seen from other players, as vanilla's
/// `LastSeenMessagesTracker`.
///
/// Each signed chat message we send includes the signatures of the last
/// messages we've seen, and tells the server which messages in its window we
/// saw, so the server can validate the chain of messages.
#[derive(Clone, Component, Debug, Default)]
pub struct LastSeenMessagesTracker {
    entries: [Option<LastSeenEntry>; LAST_SEEN_MESSAGES_MAX_LENGTH],
    tail: usize,
    offset: u32,
    last_tracked: Option<MessageSignature>,
}

#[derive(Clone, Debug)]
struct LastSeenEntry {
    signature: MessageSignature,
    /// Whether this message has been seen but not yet acknowledged.
    pending: bool,
}

impl LastSeenMessagesTracker {
    /// Track a signed message that we received. Returns false if it was the
    /// same as the last message tracked, in which case it's ignored.
    pub fn add_pending(&mut self, signature: MessageSignature) -> bool {
        if self.last_tracked.as_ref() == Some(&signature) {
            return false;
        }
        self.last_tracked = Some(signature.clone());
        self.entries[self.tail] = Some(LastSeenEntry {
            signature,
            pending: true,
        });
        self.tail = (self.tail + 1) % LAST_SEEN_MESSAGES_MAX_LENGTH;
        self.offset += 1;
        true
    }

    /// Stop tracking a message that the server deleted before we acknowledged
    /// it.
    pub fn ignore_pending(&mut self, signature: &MessageSignature) {
        for entry in &mut self.entries {
            if entry
                .as_ref()
                .is_some_and(|e| e.pending && &e.signature == signature)
            {
                *entry = None;
                return;
            }
        }
    }

    /// The number of messages received since the last acknowledgement.
    pub fn offset(&self) -> u32 {
        self.offset
    }

    /// Reset the offset to 0 and return what it was, for a
    /// `ServerboundChatAck`.
    pub fn get_and_clear_offset(&mut self) -> u32 {
        std::mem::take(&mut self.offset)
    }

    /// Acknowledge every tracked message. Returns their signatures, oldest
    /// first, for signing our message, and the update to send with it.
    pub fn generate_and_apply_update(&mut self) -> (Vec<MessageSignature>, LastSeenMessagesUpdate) {
        let offset = self.get_and_clear_offset();
        let mut acknowledged = FixedBitSet::<LAST_SEEN_MESSAGES_MAX_LENGTH>::new();
        let mut seen = Vec::new();
        for i in 0..LAST_SEEN_MESSAGES_MAX_LENGTH {
            let index = (self.tail + i) % LAST_SEEN_MESSAGES_MAX_LENGTH;
            if let Some(entry) = &mut self.entries[index] {
                acknowledged.set(i);
                seen.push(entry.signature.clone());
                entry.pending = false;
            }
        }
        let checksum = last_seen_checksum(&seen);
        (
            seen,
            LastSeenMessagesUpdate {
                offset,
                acknowledged,
                checksum,
            },
        )
    }
}

/// Vanilla's `LastSeenMessages.computeChecksum`.
pub fn last_seen_checksum(signatures: &[MessageSignature]) -> u8 {
    let checksum = signatures.iter().fold(1i32, |checksum, signature| {
        checksum
            .wrapping_mul(31)
            .wrapping_add(signature_checksum(signature))
    });
    // 0 means that there's no checksum
    match checksum as u8 {
        0 => 1,
        checksum => checksum,
    }
}

/// Vanilla's `MessageSignature.checksum`, which is Java's `Arrays.hashCode` of
/// the signature's bytes.
fn signature_checksum(signature: &MessageSignature) -> i32 {
    signature.bytes.iter().fold(1i32, |hash, &byte| {
        hash.wrapping_mul(31).wrapping_add(byte as i8 as i32)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sig(n: u8) -> MessageSignature {
        MessageSignature { bytes: [n; 256] }
    }

    #[test]
    fn tracks_the_last_twenty_in_order() {
        let mut tracker = LastSeenMessagesTracker::default();
        for n in 1..=25 {
            assert!(tracker.add_pending(sig(n)));
        }
        // the same message twice is only counted once
        assert!(!tracker.add_pending(sig(25)));

        let (seen, update) = tracker.generate_and_apply_update();
        assert_eq!(update.offset, 25);
        assert_eq!(seen.len(), 20);
        assert_eq!(seen[0], sig(6));
        assert_eq!(seen[19], sig(25));
        assert!((0..20).all(|i| update.acknowledged.index(i)));

        // already acknowledged, so there's no offset but the window is the same
        let (seen, update) = tracker.generate_and_apply_update();
        assert_eq!((update.offset, seen.len()), (0, 20));
    }

    #[test]
    fn deleted_messages_leave_the_window() {
        let mut tracker = LastSeenMessagesTracker::default();
        tracker.add_pending(sig(1));
        tracker.add_pending(sig(2));
        tracker.ignore_pending(&sig(1));
        let (seen, update) = tracker.generate_and_apply_update();
        assert_eq!(seen, vec![sig(2)]);
        // it was still received
        assert_eq!(update.offset, 2);

        // once acknowledged, deleting it doesn't remove it
        tracker.ignore_pending(&sig(2));
        assert_eq!(tracker.generate_and_apply_update().0, vec![sig(2)]);
    }

    #[test]
    fn checksums_match_java() {
        // Arrays.hashCode(new byte[]{1, -1}) == (31 + 1) * 31 - 1
        let mut bytes = [0; 256];
        bytes[254] = 1;
        bytes[255] = 255;
        let leading = (0..254).fold(1i32, |h, _| h.wrapping_mul(31));
        let expected = leading.wrapping_mul(31).wrapping_add(1).wrapping_mul(31) - 1;
        assert_eq!(signature_checksum(&MessageSignature { bytes }), expected);

        assert_eq!(last_seen_checksum(&[]), 1);
    }
}
