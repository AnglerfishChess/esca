//! Starting the scripted engine double of `tests/fixtures/`, for the tests of
//! both clients.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use esca::uci::Launch;

/// How long a double is given to answer: a bound on the machine, not on the
/// engine, since starting an interpreter on a loaded runner takes as long as
/// it takes. A case that is about a wait running out names its own short
/// budget instead.
pub const TIMEOUT: Duration = Duration::from_secs(60);

/// A Chess960 endgame: the white king on b1 with its own rook beside it on c1.
pub const BESIDE_ROOK: &str = "4k3/8/8/8/8/8/8/1KR5 w C - 0 1";

/// The interpreter that runs the engine doubles.
pub fn python() -> &'static str {
    static NAMES: [&str; 3] = ["python3", "python", "py"];
    for name in NAMES {
        let answered = Command::new(name)
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success());
        if answered {
            return name;
        }
    }
    panic!("no Python interpreter on PATH: tried {NAMES:?}");
}

/// The path of one file under `tests/fixtures/`.
pub fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// The double, misbehaving as `flags` ask, ready to be started.
pub fn launch(flags: &[&str]) -> Launch {
    let mut launch = Launch::new(python())
        .arg(fixture("fake_engine.py"))
        .timeout(TIMEOUT);
    for flag in flags {
        launch = launch.arg(flag);
    }
    launch
}

/// The same double, logging every command it is sent to `path`.
pub fn launch_logging(path: &Path, flags: &[&str]) -> Launch {
    launch(flags).arg(format!("--log={}", path.display()))
}

/// The commands a double wrote to its log, in order.
pub fn log_of(path: &Path) -> Vec<String> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(str::to_owned)
        .collect()
}

/// A fresh path for one test's command log.
pub fn log_path(name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!("esca-uci-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("a writable temporary directory");
    let path = directory.join(name);
    let _ = std::fs::remove_file(&path);
    path
}

/// The engines to try: the well-known ones on PATH, and this workspace's own
/// build of `anglerfish`.
pub fn real_engines() -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = ["stockfish", "lc0", "anglerfry", "anglerfish"]
        .iter()
        .map(PathBuf::from)
        .collect();
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../target");
    for profile in ["release", "debug"] {
        let built = workspace.join(profile).join("anglerfish");
        if built.exists() {
            found.push(built);
        }
    }
    found
}
