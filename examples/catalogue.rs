//! Rewrites `docs/features.md` from the facts catalogue.
//!
//! ```text
//! cargo run --example catalogue
//! ```

use std::fs;
use std::path::Path;

fn main() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/features.md");
    fs::write(&path, esca::facts::catalogue::markdown()).expect("docs/features.md is writable");
    println!("wrote {}", path.display());
}
