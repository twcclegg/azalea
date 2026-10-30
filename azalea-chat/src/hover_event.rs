use serde::Serialize;
#[cfg(feature = "simdnbt")]
use simdnbt::owned::NbtCompound;

use crate::FormattedText;

/// What a component shows when hovered, as vanilla's `HoverEvent`.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "action")]
pub enum HoverEvent {
    ShowText {
        value: Box<FormattedText>,
    },
    /// An item's tooltip: its id ("minecraft:diamond_sword") and count. Its
    /// data components aren't read.
    ShowItem {
        id: String,
        count: i32,
    },
    /// An entity's: its type ("minecraft:pig"), UUID (hyphenated hex) and name,
    /// if given.
    ShowEntity {
        id: String,
        uuid: String,
        name: Option<Box<FormattedText>>,
    },
}

impl HoverEvent {
    /// Reads a hover event as the game writes it in NBT (`HoverEvent.CODEC`: an
    /// "action" and that action's fields). None for an unknown action or a
    /// missing field.
    #[cfg(feature = "simdnbt")]
    pub fn from_compound(compound: &simdnbt::borrow::NbtCompound) -> Option<Self> {
        use simdnbt::FromNbtTag;
        Some(match compound.string("action")?.to_str().as_ref() {
            "show_text" => HoverEvent::ShowText {
                value: Box::new(FormattedText::from_nbt_tag(compound.get("value")?)?),
            },
            "show_item" => HoverEvent::ShowItem {
                id: compound.string("id")?.to_string(),
                count: compound.int("count").unwrap_or(1),
            },
            "show_entity" => HoverEvent::ShowEntity {
                id: compound.string("id")?.to_string(),
                // An int array of four (UUIDUtil.CODEC), or a string
                // (LENIENT_CODEC).
                uuid: match compound.int_array("uuid") {
                    Some(a) if a.len() == 4 => {
                        let hex: String = a.iter().map(|v| format!("{:08x}", *v as u32)).collect();
                        format!(
                            "{}-{}-{}-{}-{}",
                            &hex[0..8],
                            &hex[8..12],
                            &hex[12..16],
                            &hex[16..20],
                            &hex[20..32]
                        )
                    }
                    _ => compound.string("uuid")?.to_string(),
                },
                name: compound
                    .get("name")
                    .and_then(FormattedText::from_nbt_tag)
                    .map(Box::new),
            },
            _ => return None,
        })
    }
}

#[cfg(feature = "simdnbt")]
impl simdnbt::Serialize for HoverEvent {
    fn to_compound(self) -> NbtCompound {
        let mut compound = NbtCompound::new();
        let mut action = |s: &str| {
            compound.insert("action", s);
        };
        match self {
            HoverEvent::ShowText { value } => {
                action("show_text");
                compound.insert("value", value.to_compound());
            }
            HoverEvent::ShowItem { id, count } => {
                action("show_item");
                compound.insert("id", id);
                compound.insert("count", count);
            }
            HoverEvent::ShowEntity { id, uuid, name } => {
                action("show_entity");
                compound.insert("id", id);
                compound.insert("uuid", uuid);
                if let Some(name) = name {
                    compound.insert("name", name.to_compound());
                }
            }
        }
        compound
    }
}
