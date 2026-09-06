//! The Python extension module, `esca._esca`.
//!
//! Squares, roles, colours and files are text on this surface; every fact that
//! differs between the two sides is an `esca.ByColour`.

mod board;
mod convert;
mod endings;
mod explain;
mod facts;
#[cfg(feature = "openings")]
mod openings;
#[cfg(feature = "pgn")]
mod pgn;
#[cfg(feature = "polyglot")]
mod polyglot;
#[cfg(feature = "uci")]
mod uci;
mod values;

use pyo3::prelude::*;

use crate::variant::{chess960, classic};

use board::PyVariant;

/// Classic chess, the default wherever a variant is optional.
fn default_variant() -> PyVariant {
    PyVariant::new(classic())
}

#[pymodule]
#[pyo3(name = "_esca")]
fn esca_module(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<board::PyVariant>()?;
    module.add_class::<board::PySquareSet>()?;
    module.add_class::<values::PyFileSet>()?;
    module.add_class::<values::PyByColour>()?;
    module.add_class::<board::PyMove>()?;
    module.add_class::<board::PyPosition>()?;
    module.add_class::<board::PyGame>()?;
    module.add_class::<explain::PyCastling>()?;
    module.add_class::<explain::PyEnPassant>()?;
    module.add_class::<explain::PyEpCapture>()?;
    module.add_class::<explain::PyEpObstacle>()?;
    module.add_class::<explain::PyPin>()?;
    module.add_class::<explain::PySkewer>()?;
    module.add_class::<explain::PyRepetition>()?;
    module.add_class::<explain::PyNearMiss>()?;
    module.add_class::<explain::PyFiftyMove>()?;
    module.add_class::<explain::PyReset>()?;
    module.add_class::<explain::PyDrawStatus>()?;
    module.add_class::<explain::PyAutomaticDraw>()?;
    module.add_class::<explain::PyClaimableDraw>()?;
    module.add_class::<explain::PyStalemateDetail>()?;
    module.add_class::<explain::PyStuck>()?;
    endings::register(module)?;
    module.add_class::<facts::PyFacts>()?;
    module.add_class::<facts::PyPlacementFacts>()?;
    module.add_class::<facts::PyStateFacts>()?;
    module.add_class::<facts::PyHistoryFacts>()?;
    module.add_class::<facts::PyMaterialFacts>()?;
    module.add_class::<facts::PyPawnFacts>()?;
    module.add_class::<facts::PyPieceFacts>()?;
    module.add_class::<facts::PyKingFacts>()?;
    module.add_class::<facts::PyMobilityFacts>()?;
    module.add_class::<facts::PyAttackFacts>()?;
    module.add_class::<facts::PyExchangeFacts>()?;
    module.add_class::<facts::PyThreatFacts>()?;
    module.add_class::<facts::PyTacticsFacts>()?;
    module.add_class::<facts::PyEndgameFacts>()?;
    module.add_class::<facts::PyMapFacts>()?;
    module.add_class::<facts::PyMoveFacts>()?;
    module.add_class::<facts::PyAnnotatedMove>()?;

    module.add_function(wrap_pyfunction!(facts::facts_group, module)?)?;
    module.add_function(wrap_pyfunction!(facts::facts_catalogue, module)?)?;
    module.add_function(wrap_pyfunction!(facts::move_facts_catalogue, module)?)?;
    module.add_function(wrap_pyfunction!(facts::facts_layout, module)?)?;

    #[cfg(feature = "pgn")]
    {
        module.add_class::<pgn::PyPgnGame>()?;
        module.add_class::<pgn::PyPgnNode>()?;
        module.add_class::<pgn::PyPgnReader>()?;
        module.add_function(wrap_pyfunction!(pgn::pgn_read, module)?)?;
        module.add_function(wrap_pyfunction!(pgn::pgn_read_string, module)?)?;
        module.add_function(wrap_pyfunction!(pgn::pgn_count, module)?)?;
    }

    #[cfg(feature = "polyglot")]
    polyglot::register(module)?;

    #[cfg(feature = "openings")]
    openings::register(module)?;

    #[cfg(feature = "uci")]
    uci::register(module)?;

    module.add("CLASSIC", PyVariant::new(classic()))?;
    module.add("CHESS960", PyVariant::new(chess960()))?;
    module.add("KING_TO_ROOK", "king_to_rook")?;
    module.add("KING_TWO_SQUARES", "king_two_squares")?;
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
