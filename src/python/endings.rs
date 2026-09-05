//! Named endings: the material, the ending it is, the result and the method.
//!
//! The three enums are objects here rather than bare strings, because each
//! carries its own `describe()`: one compares equal to its `snake_case` name,
//! and `str()` of it is that name.

use pyo3::prelude::*;
use pyo3::types::PyString;

use crate::endings::{
    self, Bishops, Class, Ending, Evidence, PawnRace, Signature, Technique, Verdict,
};

use super::board::PyPosition;
use super::convert::{colour_from, colour_name, role_from, square_name};

/// The Python hash of `name`, so a value and the name it compares equal to
/// hash alike.
fn name_hash(py: Python<'_>, name: &str) -> PyResult<isize> {
    PyString::new(py, name).hash()
}

/// Whether `other` is `name` or a value of the same case.
fn equals<T: PartialEq + for<'a, 'py> FromPyObject<'a, 'py>>(
    mine: &T,
    name: &str,
    other: &Bound<'_, PyAny>,
) -> bool {
    if let Ok(text) = other.extract::<String>() {
        return text == name;
    }
    other.extract::<T>().is_ok_and(|theirs| theirs == *mine)
}

/// One named ending of the catalogue.
#[pyclass(frozen, from_py_object, module = "esca.endings", name = "EndingClass")]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PyEndingClass {
    inner: Class,
}

#[pymethods]
impl PyEndingClass {
    /// Every class, in the order the catalogue lists them.
    #[staticmethod]
    fn all() -> Vec<PyEndingClass> {
        Class::ALL
            .into_iter()
            .map(|inner| PyEndingClass { inner })
            .collect()
    }

    /// The name in `snake_case`, which the class compares equal to.
    #[getter]
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    /// One plain sentence naming the ending.
    fn describe(&self) -> &'static str {
        self.inner.describe()
    }

    fn __str__(&self) -> &'static str {
        self.inner.name()
    }

    fn __repr__(&self) -> String {
        format!("<EndingClass {}>", self.inner.name())
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        equals(self, self.inner.name(), other)
    }

    fn __hash__(&self, py: Python<'_>) -> PyResult<isize> {
        name_hash(py, self.inner.name())
    }
}

/// What theory says the result of an ending is.
#[pyclass(
    frozen,
    from_py_object,
    module = "esca.endings",
    name = "EndingVerdict"
)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PyEndingVerdict {
    inner: Verdict,
}

#[pymethods]
impl PyEndingVerdict {
    /// `win`, `usually_win`, `usually_draw`, `draw` or `unknown`, which the
    /// verdict compares equal to.
    #[getter]
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    /// The colour the verdict names, if it names one.
    #[getter]
    fn winner(&self) -> Option<String> {
        self.inner.winner().map(colour_name)
    }

    /// One plain sentence naming the result.
    fn describe(&self) -> String {
        self.inner.describe()
    }

    fn __str__(&self) -> &'static str {
        self.inner.name()
    }

    fn __repr__(&self) -> String {
        format!("<EndingVerdict {}>", self.inner.name())
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        equals(self, self.inner.name(), other)
    }

    fn __hash__(&self, py: Python<'_>) -> PyResult<isize> {
        name_hash(py, self.inner.name())
    }
}

/// The named method an ending is played by.
#[pyclass(
    frozen,
    from_py_object,
    module = "esca.endings",
    name = "EndingTechnique"
)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PyEndingTechnique {
    inner: Technique,
}

#[pymethods]
impl PyEndingTechnique {
    /// Every technique, in the order the catalogue lists them.
    #[staticmethod]
    fn all() -> Vec<PyEndingTechnique> {
        Technique::ALL
            .into_iter()
            .map(|inner| PyEndingTechnique { inner })
            .collect()
    }

    /// The name in `snake_case`, which the technique compares equal to.
    #[getter]
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    /// One plain sentence naming the method and how it is played.
    fn describe(&self) -> &'static str {
        self.inner.describe()
    }

    fn __str__(&self) -> &'static str {
        self.inner.name()
    }

    fn __repr__(&self) -> String {
        format!("<EndingTechnique {}>", self.inner.name())
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        equals(self, self.inner.name(), other)
    }

    fn __hash__(&self, py: Python<'_>) -> PyResult<isize> {
        name_hash(py, self.inner.name())
    }
}

/// The material both sides hold, written the way endings are named.
#[pyclass(
    frozen,
    skip_from_py_object,
    module = "esca.endings",
    name = "MaterialSignature"
)]
#[derive(Clone)]
pub struct PyMaterialSignature {
    inner: Signature,
}

#[pymethods]
impl PyMaterialSignature {
    /// The canonical spelling, stronger side first: `KRPvKR`.
    #[getter]
    fn text(&self) -> &str {
        &self.inner.text
    }

    /// The side written first.
    #[getter]
    fn stronger(&self) -> String {
        colour_name(self.inner.stronger)
    }

    /// The pawns of both sides.
    #[getter]
    fn pawns(&self) -> u32 {
        self.inner.pawns()
    }

    /// How many units of `role` `colour` has.
    fn count(&self, colour: &str, role: &str) -> PyResult<u8> {
        Ok(self.inner.count(colour_from(colour)?, role_from(role)?))
    }

    /// The pieces of `colour`: everything that is neither a king nor a pawn.
    fn pieces(&self, colour: &str) -> PyResult<u32> {
        Ok(self.inner.pieces(colour_from(colour)?))
    }

    /// The conventional material of `colour`, the king counting nothing.
    fn value(&self, colour: &str) -> PyResult<u32> {
        Ok(self.inner.value[colour_from(colour)?.index()])
    }

    /// One plain sentence naming what each side has.
    fn describe(&self) -> String {
        self.inner.describe()
    }

    fn __str__(&self) -> &str {
        &self.inner.text
    }

    fn __repr__(&self) -> String {
        format!("<MaterialSignature {}>", self.inner.text)
    }
}

/// The race of the only pawn on the board.
#[pyclass(
    frozen,
    skip_from_py_object,
    module = "esca.endings",
    name = "PawnRace"
)]
#[derive(Clone)]
pub struct PyPawnRace {
    /// Where the pawn stands.
    #[pyo3(get)]
    pawn: String,
    /// Whose pawn it is.
    #[pyo3(get)]
    colour: String,
    /// The square it promotes on.
    #[pyo3(get)]
    promotion: String,
    /// It stands on the a- or the h-file.
    #[pyo3(get)]
    rook_pawn: bool,
    /// Pawn moves left to promotion, a double first step counted as one.
    #[pyo3(get)]
    steps: u32,
    /// The defending king reaches the promotion square no later than the pawn.
    #[pyo3(get)]
    defender_inside_square: bool,
    /// The pawn's own king stands on the pawn's file, ahead of the pawn.
    #[pyo3(get)]
    attacker_in_front: bool,
    /// The defending king stands on the pawn's file, ahead of the pawn.
    #[pyo3(get)]
    defender_in_front: bool,
    /// The defending king stands on the promotion square or beside it.
    #[pyo3(get)]
    defender_holds_the_corner: bool,
    race: PawnRace,
}

impl PyPawnRace {
    fn of(race: PawnRace) -> PyPawnRace {
        PyPawnRace {
            pawn: square_name(race.pawn),
            colour: colour_name(race.colour),
            promotion: square_name(race.promotion),
            rook_pawn: race.rook_pawn,
            steps: race.steps,
            defender_inside_square: race.defender_inside_square,
            attacker_in_front: race.attacker_in_front,
            defender_in_front: race.defender_in_front,
            defender_holds_the_corner: race.defender_holds_the_corner,
            race,
        }
    }
}

#[pymethods]
impl PyPawnRace {
    /// One plain sentence: whose pawn is running where, and whether the other
    /// king catches it.
    fn describe(&self) -> String {
        self.race.describe()
    }

    fn __repr__(&self) -> String {
        format!("<PawnRace {}>", self.pawn)
    }
}

/// The bishops on the board, when at least one stands on it.
#[pyclass(frozen, skip_from_py_object, module = "esca.endings", name = "Bishops")]
#[derive(Clone)]
pub struct PyBishops {
    /// One bishop each, on opposite square colours.
    #[pyo3(get)]
    opposite_colours: bool,
    /// Every bishop on the board stands on one square colour.
    #[pyo3(get)]
    same_colour: bool,
    /// The only pawn is a rook pawn its own side's bishops cannot cover the
    /// promotion square of.
    #[pyo3(get)]
    wrong_bishop: bool,
    bishops: Bishops,
}

impl PyBishops {
    fn of(bishops: Bishops) -> PyBishops {
        PyBishops {
            opposite_colours: bishops.opposite_colours,
            same_colour: bishops.same_colour,
            wrong_bishop: bishops.wrong_bishop,
            bishops,
        }
    }
}

#[pymethods]
impl PyBishops {
    /// One plain sentence naming what the bishops can and cannot reach.
    fn describe(&self) -> String {
        self.bishops.describe()
    }

    fn __repr__(&self) -> String {
        format!("<Bishops opposite_colours={}>", self.opposite_colours)
    }
}

/// The position-specific facts an ending's verdict and technique are read off.
#[pyclass(
    frozen,
    skip_from_py_object,
    module = "esca.endings",
    name = "EndingEvidence"
)]
#[derive(Clone)]
pub struct PyEndingEvidence {
    /// The race, when exactly one pawn stands on the board.
    #[pyo3(get)]
    pawn: Option<PyPawnRace>,
    /// The bishops, when at least one stands on the board.
    #[pyo3(get)]
    bishops: Option<PyBishops>,
    /// The kings stand in opposition.
    #[pyo3(get)]
    opposition: bool,
    evidence: Evidence,
}

impl PyEndingEvidence {
    fn of(evidence: Evidence) -> PyEndingEvidence {
        PyEndingEvidence {
            pawn: evidence.pawn.map(PyPawnRace::of),
            bishops: evidence.bishops.map(PyBishops::of),
            opposition: evidence.opposition,
            evidence,
        }
    }
}

#[pymethods]
impl PyEndingEvidence {
    /// One plain sentence per group that applies.
    fn describe(&self) -> String {
        self.evidence.describe()
    }

    fn __repr__(&self) -> String {
        format!("<EndingEvidence opposition={}>", self.opposition)
    }
}

/// What ending a position is, and what is known about it.
#[pyclass(frozen, skip_from_py_object, module = "esca.endings", name = "Ending")]
#[derive(Clone)]
pub struct PyEnding {
    /// The named ending.
    #[pyo3(get)]
    class_: PyEndingClass,
    /// The material both sides hold.
    #[pyo3(get)]
    signature: PyMaterialSignature,
    /// The result.
    #[pyo3(get)]
    verdict: PyEndingVerdict,
    /// The method the ending is played by.
    #[pyo3(get)]
    technique: PyEndingTechnique,
    /// The position-specific facts behind the verdict and the technique.
    #[pyo3(get)]
    evidence: PyEndingEvidence,
    ending: Ending,
}

impl PyEnding {
    pub(crate) fn of(ending: Ending) -> PyEnding {
        PyEnding {
            class_: PyEndingClass {
                inner: ending.class,
            },
            signature: PyMaterialSignature {
                inner: ending.signature.clone(),
            },
            verdict: PyEndingVerdict {
                inner: ending.verdict,
            },
            technique: PyEndingTechnique {
                inner: ending.technique,
            },
            evidence: PyEndingEvidence::of(ending.evidence),
            ending,
        }
    }
}

#[pymethods]
impl PyEnding {
    /// The material, the ending, the result and the method, in plain
    /// sentences.
    fn describe(&self) -> String {
        self.ending.describe()
    }

    fn __repr__(&self) -> String {
        format!("<Ending {}>", self.ending.class.name())
    }
}

/// The ending `position` is.
#[pyfunction]
pub(crate) fn endings_classify(position: &PyPosition) -> PyEnding {
    PyEnding::of(endings::classify(&position.inner))
}

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyEnding>()?;
    module.add_class::<PyEndingClass>()?;
    module.add_class::<PyEndingVerdict>()?;
    module.add_class::<PyEndingTechnique>()?;
    module.add_class::<PyMaterialSignature>()?;
    module.add_class::<PyEndingEvidence>()?;
    module.add_class::<PyPawnRace>()?;
    module.add_class::<PyBishops>()?;
    module.add_function(wrap_pyfunction!(endings_classify, module)?)?;
    Ok(())
}
