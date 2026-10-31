//! Finding the arguments of a command that have to be signed, like the
//! message in `/msg <targets> <message>`.

use azalea_protocol::packets::game::c_commands::{
    BrigadierNodeStub, BrigadierParser, BrigadierString, NodeType,
};
use bevy_ecs::prelude::*;

/// The command tree that the server sent us in `ClientboundCommands`.
#[derive(Clone, Component, Debug, Default)]
pub struct ServerCommandTree {
    pub nodes: Vec<BrigadierNodeStub>,
    pub root_index: usize,
}

impl ServerCommandTree {
    /// The arguments of `command` (without the leading slash) that vanilla
    /// signs, which are the ones parsed as `message`. Returns each argument's
    /// name and value.
    ///
    /// Vanilla parses the command with Brigadier to find them. This follows
    /// the tree in the same way without parsing every argument type: a word
    /// matching a literal goes that way, and otherwise each argument is tried
    /// in order, keeping the first that reads the whole command. Selectors,
    /// quoted strings and coordinates are read as one argument, and numbers
    /// and booleans are checked so a word that can't be one tries the next
    /// argument.
    pub fn signable_arguments(&self, command: &str) -> Vec<(String, String)> {
        // limits how much work a pathological tree can cause
        let mut budget = 4096;
        self.walk(self.root_index, command, &mut budget)
            .unwrap_or_default()
    }

    fn walk(&self, node: usize, rest: &str, budget: &mut u32) -> Option<Vec<(String, String)>> {
        *budget = budget.checked_sub(1)?;
        if rest.is_empty() {
            return Some(Vec::new());
        }
        let here = self.nodes.get(node)?;
        // a redirect (like /tell to /msg, or /execute's run) continues from the
        // target's children
        let children = match here.redirect_node {
            Some(target) => &self.nodes.get(target as usize)?.children,
            None => &here.children,
        };
        // after a node, either the command ends or a space comes before the
        // next one
        let next = |child: u32, after: &str, budget: &mut u32| {
            let after = if after.is_empty() {
                after
            } else {
                after.strip_prefix(' ')?
            };
            self.walk(child as usize, after, budget)
        };

        let word_end = rest.find(' ').unwrap_or(rest.len());
        let literal = children.iter().copied().find(|&child| {
            matches!(
                self.nodes.get(child as usize).map(|n| &n.node_type),
                Some(NodeType::Literal { name }) if *name == rest[..word_end]
            )
        });
        if let Some(child) = literal {
            return next(child, &rest[word_end..], budget);
        }

        for &child in children {
            let Some(NodeType::Argument { name, parser, .. }) =
                self.nodes.get(child as usize).map(|n| &n.node_type)
            else {
                continue;
            };
            let words = match parser {
                BrigadierParser::Message => return Some(vec![(name.clone(), rest.to_owned())]),
                BrigadierParser::String(BrigadierString::GreedyPhrase) => return Some(Vec::new()),
                BrigadierParser::BlockPos | BrigadierParser::Vec3 => 3,
                BrigadierParser::ColumnPos | BrigadierParser::Vec2 | BrigadierParser::Rotation => 2,
                _ => 1,
            };
            let end = argument_end(rest, words);
            let value = &rest[..end];
            let fits = match parser {
                BrigadierParser::Integer(_) | BrigadierParser::Long(_) => {
                    value.parse::<i64>().is_ok()
                }
                BrigadierParser::Float(_) | BrigadierParser::Double(_) => {
                    value.parse::<f64>().is_ok()
                }
                BrigadierParser::Bool => matches!(value, "true" | "false"),
                _ => true,
            };
            if fits && let Some(found) = next(child, &rest[end..], budget) {
                return Some(found);
            }
        }
        None
    }
}

/// Where an argument of `words` words ends: after that many spaces outside of
/// brackets and quotes.
fn argument_end(text: &str, mut words: u32) -> usize {
    let mut depth = 0i32;
    let mut quote = None;
    let mut escaped = false;
    for (i, c) in text.char_indices() {
        match (quote, c) {
            (Some(_), _) if escaped => escaped = false,
            (Some(_), '\\') => escaped = true,
            (Some(q), _) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(c),
            (None, '[' | '{') => depth += 1,
            (None, ']' | '}') => depth -= 1,
            (None, ' ') if depth <= 0 => {
                words -= 1;
                if words == 0 {
                    return i;
                }
            }
            _ => {}
        }
    }
    text.len()
}

#[cfg(test)]
mod tests {
    use azalea_protocol::packets::game::c_commands::{BrigadierNumber, EntityParser};

    use super::*;

    fn root(children: Vec<u32>) -> BrigadierNodeStub {
        BrigadierNodeStub {
            is_executable: false,
            children,
            redirect_node: None,
            node_type: NodeType::Root,
            is_restricted: false,
        }
    }
    fn literal(name: &str, children: Vec<u32>) -> BrigadierNodeStub {
        BrigadierNodeStub {
            is_executable: false,
            children,
            redirect_node: None,
            node_type: NodeType::Literal { name: name.into() },
            is_restricted: false,
        }
    }
    fn argument(name: &str, parser: BrigadierParser, children: Vec<u32>) -> BrigadierNodeStub {
        BrigadierNodeStub {
            is_executable: true,
            children,
            redirect_node: None,
            node_type: NodeType::Argument {
                name: name.into(),
                parser,
                suggestions_type: None,
            },
            is_restricted: false,
        }
    }
    fn players() -> BrigadierParser {
        BrigadierParser::Entity(EntityParser {
            single: false,
            players_only: true,
        })
    }
    fn message(m: &str) -> Vec<(String, String)> {
        vec![("message".to_owned(), m.to_owned())]
    }

    #[test]
    fn finds_message_arguments() {
        // msg <targets> <message>, tell -> msg, me <action>, time set <time>
        let mut tell = literal("tell", vec![]);
        tell.redirect_node = Some(1);
        let tree = ServerCommandTree {
            nodes: vec![
                root(vec![1, 4, 5, 7]),
                literal("msg", vec![2]),
                argument("targets", players(), vec![3]),
                argument("message", BrigadierParser::Message, vec![]),
                tell,
                literal("me", vec![6]),
                argument("action", BrigadierParser::Message, vec![]),
                literal("time", vec![8]),
                literal("set", vec![9]),
                argument("time", BrigadierParser::Time { min: 0 }, vec![]),
            ],
            root_index: 0,
        };
        let args = |c: &str| tree.signable_arguments(c);
        assert_eq!(args("msg Kid hello there"), message("hello there"));
        assert_eq!(args("tell @a[name=\"a b\"] hi"), message("hi"));
        assert_eq!(args("me waves"), vec![("action".into(), "waves".into())]);
        assert!(args("time set day").is_empty());
        assert!(args("tp 1 2 3").is_empty());
    }

    #[test]
    fn follows_coordinates_redirects_and_alternatives() {
        // execute (positioned <pos> | rotated <rot> | run -> root), say
        // <message>, w (<targets> <count> | <name> <message>)
        let mut positioned_pos = argument("pos", BrigadierParser::Vec3, vec![]);
        positioned_pos.redirect_node = Some(1);
        let mut rotated_rot = argument("rot", BrigadierParser::Rotation, vec![]);
        rotated_rot.redirect_node = Some(1);
        let mut run = literal("run", vec![]);
        run.redirect_node = Some(0);
        let tree = ServerCommandTree {
            nodes: vec![
                root(vec![1, 8, 10]),
                literal("execute", vec![2, 4, 6]),
                literal("positioned", vec![3]),
                positioned_pos,
                literal("rotated", vec![5]),
                rotated_rot,
                run,
                argument("unused", BrigadierParser::Bool, vec![]),
                literal("say", vec![9]),
                argument("message", BrigadierParser::Message, vec![]),
                literal("w", vec![11, 13]),
                argument("targets", players(), vec![12]),
                argument(
                    "count",
                    BrigadierParser::Integer(BrigadierNumber {
                        min: None,
                        max: None,
                    }),
                    vec![],
                ),
                argument(
                    "name",
                    BrigadierParser::String(BrigadierString::SingleWord),
                    vec![14],
                ),
                argument("message", BrigadierParser::Message, vec![]),
            ],
            root_index: 0,
        };
        let args = |c: &str| tree.signable_arguments(c);
        assert_eq!(args("execute positioned 1 64 1 run say hi"), message("hi"));
        assert_eq!(
            args("execute positioned ^ ^1 ^ rotated 0 90 run say hi there"),
            message("hi there")
        );
        // the first alternative reads the whole command, and has no message
        assert_eq!(args("w Kid 3"), vec![]);
        // "hi" isn't a count, so the second alternative is used
        assert_eq!(args("w Kid hi"), message("hi"));
        assert_eq!(args("w Kid hi there"), message("hi there"));
        // an escaped quote stays in the string
        assert_eq!(args("w \"a \\\" b\" hi"), message("hi"));
        // two words aren't a position
        assert!(args("execute positioned 1 64 run say hi").is_empty());
        assert!(args("execute run").is_empty());
    }
}
