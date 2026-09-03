//! The Python extension module, `esca._esca`.
//!
//! Squares, roles, colours and file sets are text on this surface; every
//! side-paired fact is a pair indexed by `esca.US` and `esca.THEM`.

mod board;
mod convert;
mod encode;
mod facts;
#[cfg(feature = "lichess")]
mod lichess;
#[cfg(feature = "pgn")]
mod pgn;
#[cfg(feature = "uci")]
mod uci;

use pyo3::prelude::*;

use crate::facts::MoveFacts;
use crate::schema::Schema;
use crate::variant::{chess960, classic};

use board::PyVariant;
use encode::{PyMoveSchema, PySchema};

/// Classic chess, the default wherever a variant is optional.
fn default_variant() -> PyVariant {
    PyVariant::new(classic())
}

/// The v1 schema, the default wherever a schema is optional.
fn default_schema() -> PySchema {
    PySchema::new(Schema::v1())
}

fn default_move_schema() -> PyMoveSchema {
    PyMoveSchema::new(Schema::v1().moves())
}

#[pymodule]
#[pyo3(name = "_esca")]
fn esca_module(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<board::PyVariant>()?;
    module.add_class::<board::PySquareSet>()?;
    module.add_class::<board::PyMove>()?;
    module.add_class::<board::PyPosition>()?;
    module.add_class::<board::PyGame>()?;
    module.add_class::<encode::PySchema>()?;
    module.add_class::<encode::PyMoveSchema>()?;
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
    module.add_class::<facts::PyPlaneFacts>()?;
    module.add_class::<facts::PyMoveFacts>()?;
    module.add_class::<facts::PyAnnotatedMove>()?;

    module.add_function(wrap_pyfunction!(encode::encode, module)?)?;
    module.add_function(wrap_pyfunction!(encode::encode_into, module)?)?;
    module.add_function(wrap_pyfunction!(encode::encode_moves, module)?)?;
    module.add_function(wrap_pyfunction!(encode::features_for, module)?)?;
    module.add_function(wrap_pyfunction!(encode::schema, module)?)?;
    module.add_function(wrap_pyfunction!(facts::facts_group, module)?)?;

    #[cfg(feature = "lichess")]
    {
        module.add_class::<lichess::PyBatch>()?;
        module.add_class::<lichess::PyBatches>()?;
        module.add_function(wrap_pyfunction!(lichess::batches, module)?)?;
    }

    #[cfg(feature = "pgn")]
    {
        module.add_class::<pgn::PyPgnGame>()?;
        module.add_class::<pgn::PyPgnNode>()?;
        module.add_class::<pgn::PyPgnReader>()?;
        module.add_function(wrap_pyfunction!(pgn::pgn_read, module)?)?;
        module.add_function(wrap_pyfunction!(pgn::pgn_read_string, module)?)?;
        module.add_function(wrap_pyfunction!(pgn::pgn_count, module)?)?;
    }

    #[cfg(feature = "uci")]
    uci::register(module)?;

    module.add("CLASSIC", PyVariant::new(classic()))?;
    module.add("CHESS960", PyVariant::new(chess960()))?;
    module.add("US", 0usize)?;
    module.add("THEM", 1usize)?;
    module.add("KING_TO_ROOK", "king_to_rook")?;
    module.add("KING_TWO_SQUARES", "king_two_squares")?;
    module.add("SCHEMA_V1", default_schema())?;
    module.add("SCHEMA", default_schema())?;
    module.add("SCHEMA_ID", Schema::v1().id().to_string())?;
    module.add("WIDTH", Schema::v1().width())?;
    module.add("MOVE_WIDTH", MoveFacts::WIDTH)?;
    module.add("MOVE_SCHEMA", default_move_schema())?;
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
