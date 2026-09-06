//! Export-format PGN text.

use super::{Game, Node};

/// The longest movetext line export format writes. A token longer than this
/// stands alone on its line and overruns it.
pub const EXPORT_WIDTH: usize = 80;

/// The export-format text of `game`.
pub(crate) fn game(game: &Game) -> String {
    let mut out = String::new();
    for (name, value) in game.headers.export_order() {
        out.push_str(&format!("[{name} \"{}\"]\n", escape(value)));
    }
    if !game.headers.is_empty() {
        out.push('\n');
    }

    let mut tokens = Tokens::new();
    tokens.comment(&game.comment);
    let (fullmove, white) = game.numbering();
    line(&game.moves, fullmove, white, true, &mut tokens);
    tokens.push(game.result.as_str().to_string());

    for row in wrap(&tokens.items, EXPORT_WIDTH) {
        out.push_str(&row);
        out.push('\n');
    }
    out
}

/// A tag value with `\` and `"` escaped.
fn escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        if c == '\\' || c == '"' {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// The movetext tokens, each of which stays whole on one line.
struct Tokens {
    items: Vec<String>,
}

impl Tokens {
    fn new() -> Tokens {
        Tokens { items: Vec::new() }
    }

    fn push(&mut self, token: String) {
        self.items.push(token);
    }

    /// Adds `suffix` to the back of the last token.
    fn suffix(&mut self, suffix: &str) {
        match self.items.last_mut() {
            Some(last) => last.push_str(suffix),
            None => self.items.push(suffix.to_string()),
        }
    }

    /// A `{}` comment, its words broken so a long one still wraps.
    fn comment(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        let words: Vec<&str> = text.split_whitespace().collect();
        match words.split_first() {
            None => self.push("{}".to_string()),
            Some((first, rest)) => {
                self.push(format!("{{{first}"));
                for word in rest {
                    self.push((*word).to_string());
                }
                self.suffix("}");
            }
        }
    }
}

/// Writes one line of the tree. `white` is the side to move at its first
/// move, `fullmove` that move's number, and `number` whether a black move
/// there still needs its `12...`.
fn line(nodes: &[Node], fullmove: u32, white: bool, number: bool, out: &mut Tokens) {
    let mut fullmove = fullmove;
    let mut white = white;
    let mut number = number;
    for node in nodes {
        if !node.comment_before.is_empty() {
            out.comment(&node.comment_before);
            number = true;
        }
        let text = if white {
            format!("{fullmove}. {}", node.san)
        } else if number {
            format!("{fullmove}... {}", node.san)
        } else {
            node.san.clone()
        };
        out.push(text);
        number = false;
        for nag in &node.nags {
            out.push(format!("${nag}"));
        }
        if !node.comment_after.is_empty() {
            out.comment(&node.comment_after);
            number = true;
        }
        for variation in &node.variations {
            let before = out.items.len();
            line(variation, fullmove, white, true, out);
            if out.items.len() == before {
                out.push("()".to_string());
            } else {
                let first = out.items[before].clone();
                out.items[before] = format!("({first}");
                out.suffix(")");
            }
            number = true;
        }
        if !white {
            fullmove += 1;
        }
        white = !white;
    }
}

/// Greedily packs `tokens` into lines of at most `width` characters.
fn wrap(tokens: &[String], width: usize) -> Vec<String> {
    let mut rows: Vec<String> = Vec::new();
    let mut row = String::new();
    for token in tokens {
        let len = row.chars().count();
        if row.is_empty() {
            row.push_str(token);
        } else if len + 1 + token.chars().count() <= width {
            row.push(' ');
            row.push_str(token);
        } else {
            rows.push(std::mem::take(&mut row));
            row.push_str(token);
        }
    }
    if !row.is_empty() {
        rows.push(row);
    }
    rows
}
