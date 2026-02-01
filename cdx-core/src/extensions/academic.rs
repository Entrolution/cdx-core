//! Academic extension for Codex documents.
//!
//! This extension provides specialized content types for academic and
//! scientific documents including theorems, proofs, exercises, and algorithms.
//!
//! # Features
//!
//! - **Abstract**: Paper abstracts with keywords and structured sections
//! - **Theorem**: Theorem-like blocks (theorem, lemma, proposition, etc.)
//! - **Proof**: Proof blocks with method annotations
//! - **Exercise**: Exercises with hints and solutions
//! - **`ExerciseSet`**: Grouped exercises with shared context
//! - **`EquationGroup`**: Multi-line equation environments
//! - **Algorithm**: Pseudocode blocks with line numbering
//!
//! # Example
//!
//! ```json
//! {
//!   "type": "academic:theorem",
//!   "variant": "theorem",
//!   "label": "Pythagorean Theorem",
//!   "number": "3.1",
//!   "children": [...]
//! }
//! ```

use serde::{Deserialize, Serialize};

use crate::content::Block;

// ============================================================================
// Abstract
// ============================================================================

/// An academic abstract with optional keywords and structured sections.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Abstract {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Abstract content blocks.
    pub children: Vec<Block>,

    /// Keywords for the paper.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,

    /// Structured sections within the abstract (background, methods, results, conclusions).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sections: Vec<AbstractSection>,
}

impl Abstract {
    /// Create a new abstract.
    #[must_use]
    pub fn new(children: Vec<Block>) -> Self {
        Self {
            id: None,
            children,
            keywords: Vec::new(),
            sections: Vec::new(),
        }
    }

    /// Add keywords.
    #[must_use]
    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }

    /// Add a section.
    #[must_use]
    pub fn with_section(mut self, section: AbstractSection) -> Self {
        self.sections.push(section);
        self
    }
}

/// A structured section within an abstract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbstractSection {
    /// Section type.
    pub section_type: AbstractSectionType,

    /// Section content.
    pub children: Vec<Block>,
}

/// Types of structured abstract sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AbstractSectionType {
    /// Background/context.
    Background,
    /// Research objectives.
    Objectives,
    /// Methods used.
    Methods,
    /// Results obtained.
    Results,
    /// Conclusions drawn.
    Conclusions,
}

// ============================================================================
// Theorem
// ============================================================================

/// A theorem-like block (theorem, lemma, proposition, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Theorem {
    /// Optional unique identifier for cross-referencing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Type of theorem-like statement.
    pub variant: TheoremVariant,

    /// Optional label/title for the theorem.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// Theorem number (e.g., "3.1", "A.2").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,

    /// Statement content.
    pub children: Vec<Block>,

    /// Attribution (e.g., "Euclid", "Fermat").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,

    /// Citation reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub citation: Option<String>,
}

impl Theorem {
    /// Create a new theorem.
    #[must_use]
    pub fn new(variant: TheoremVariant, children: Vec<Block>) -> Self {
        Self {
            id: None,
            variant,
            label: None,
            number: None,
            children,
            attribution: None,
            citation: None,
        }
    }

    /// Set the label.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the number.
    #[must_use]
    pub fn with_number(mut self, number: impl Into<String>) -> Self {
        self.number = Some(number.into());
        self
    }

    /// Set the ID.
    #[must_use]
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set attribution.
    #[must_use]
    pub fn with_attribution(mut self, attribution: impl Into<String>) -> Self {
        self.attribution = Some(attribution.into());
        self
    }
}

/// Variant of theorem-like statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TheoremVariant {
    /// Main theorem.
    Theorem,
    /// Lemma (helper result).
    Lemma,
    /// Proposition.
    Proposition,
    /// Corollary (follows from theorem).
    Corollary,
    /// Definition.
    Definition,
    /// Conjecture (unproven).
    Conjecture,
    /// Remark.
    Remark,
    /// Example.
    Example,
    /// Axiom.
    Axiom,
    /// Claim.
    Claim,
    /// Fact.
    Fact,
    /// Assumption.
    Assumption,
}

impl std::fmt::Display for TheoremVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Theorem => write!(f, "Theorem"),
            Self::Lemma => write!(f, "Lemma"),
            Self::Proposition => write!(f, "Proposition"),
            Self::Corollary => write!(f, "Corollary"),
            Self::Definition => write!(f, "Definition"),
            Self::Conjecture => write!(f, "Conjecture"),
            Self::Remark => write!(f, "Remark"),
            Self::Example => write!(f, "Example"),
            Self::Axiom => write!(f, "Axiom"),
            Self::Claim => write!(f, "Claim"),
            Self::Fact => write!(f, "Fact"),
            Self::Assumption => write!(f, "Assumption"),
        }
    }
}

// ============================================================================
// Proof
// ============================================================================

/// A proof block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Proof {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Reference to the theorem being proved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theorem_ref: Option<String>,

    /// Proof method used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<ProofMethod>,

    /// Proof content.
    pub children: Vec<Block>,

    /// Custom QED symbol (defaults to standard square).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub qed_symbol: Option<String>,
}

impl Proof {
    /// Create a new proof.
    #[must_use]
    pub fn new(children: Vec<Block>) -> Self {
        Self {
            id: None,
            theorem_ref: None,
            method: None,
            children,
            qed_symbol: None,
        }
    }

    /// Set the theorem reference.
    #[must_use]
    pub fn of_theorem(mut self, theorem_id: impl Into<String>) -> Self {
        self.theorem_ref = Some(theorem_id.into());
        self
    }

    /// Set the proof method.
    #[must_use]
    pub fn with_method(mut self, method: ProofMethod) -> Self {
        self.method = Some(method);
        self
    }
}

/// Method of proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProofMethod {
    /// Direct proof.
    Direct,
    /// Proof by contradiction.
    Contradiction,
    /// Proof by contrapositive.
    Contrapositive,
    /// Proof by induction.
    Induction,
    /// Strong induction.
    StrongInduction,
    /// Proof by cases.
    Cases,
    /// Constructive proof.
    Constructive,
    /// Existence proof.
    Existence,
    /// Uniqueness proof.
    Uniqueness,
    /// Proof sketch.
    Sketch,
}

// ============================================================================
// Exercise
// ============================================================================

/// An exercise with optional hints and solutions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exercise {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Exercise number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,

    /// Difficulty level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<Difficulty>,

    /// Points/marks for the exercise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub points: Option<u32>,

    /// Exercise statement.
    pub children: Vec<Block>,

    /// Sub-parts of the exercise.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parts: Vec<ExercisePart>,

    /// Hints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hints: Vec<Block>,

    /// Solution (may be hidden).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub solution: Option<Solution>,
}

impl Exercise {
    /// Create a new exercise.
    #[must_use]
    pub fn new(children: Vec<Block>) -> Self {
        Self {
            id: None,
            number: None,
            difficulty: None,
            points: None,
            children,
            parts: Vec::new(),
            hints: Vec::new(),
            solution: None,
        }
    }

    /// Set the number.
    #[must_use]
    pub fn with_number(mut self, number: impl Into<String>) -> Self {
        self.number = Some(number.into());
        self
    }

    /// Set difficulty.
    #[must_use]
    pub fn with_difficulty(mut self, difficulty: Difficulty) -> Self {
        self.difficulty = Some(difficulty);
        self
    }

    /// Set points.
    #[must_use]
    pub fn with_points(mut self, points: u32) -> Self {
        self.points = Some(points);
        self
    }

    /// Add a part.
    #[must_use]
    pub fn with_part(mut self, part: ExercisePart) -> Self {
        self.parts.push(part);
        self
    }

    /// Add a hint.
    #[must_use]
    pub fn with_hint(mut self, hint: Vec<Block>) -> Self {
        self.hints.extend(hint);
        self
    }

    /// Set the solution.
    #[must_use]
    pub fn with_solution(mut self, solution: Solution) -> Self {
        self.solution = Some(solution);
        self
    }
}

/// Difficulty level for exercises.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    /// Easy difficulty.
    Easy,
    /// Medium difficulty.
    Medium,
    /// Hard difficulty.
    Hard,
    /// Challenge problem.
    Challenge,
}

/// A sub-part of an exercise.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExercisePart {
    /// Part label (a, b, c or i, ii, iii).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// Points for this part.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub points: Option<u32>,

    /// Part content.
    pub children: Vec<Block>,
}

/// A solution to an exercise.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Solution {
    /// Whether the solution should be hidden.
    #[serde(default)]
    pub hidden: bool,

    /// Solution content.
    pub children: Vec<Block>,
}

// ============================================================================
// ExerciseSet
// ============================================================================

/// A set of related exercises with shared context.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExerciseSet {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Title of the exercise set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Shared context/preamble for all exercises.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context: Vec<Block>,

    /// Exercises in the set.
    pub exercises: Vec<Exercise>,

    /// Total points for the set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_points: Option<u32>,
}

impl ExerciseSet {
    /// Create a new exercise set.
    #[must_use]
    pub fn new(exercises: Vec<Exercise>) -> Self {
        Self {
            id: None,
            title: None,
            context: Vec::new(),
            exercises,
            total_points: None,
        }
    }

    /// Set the title.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set context.
    #[must_use]
    pub fn with_context(mut self, context: Vec<Block>) -> Self {
        self.context = context;
        self
    }
}

// ============================================================================
// EquationGroup
// ============================================================================

/// A group of related equations (align, gather, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EquationGroup {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Environment type.
    pub environment: EquationEnvironment,

    /// Equations in the group.
    pub equations: Vec<Equation>,

    /// Whether to number equations.
    #[serde(default = "default_true")]
    pub numbered: bool,
}

fn default_true() -> bool {
    true
}

impl EquationGroup {
    /// Create a new equation group.
    #[must_use]
    pub fn new(environment: EquationEnvironment, equations: Vec<Equation>) -> Self {
        Self {
            id: None,
            environment,
            equations,
            numbered: true,
        }
    }

    /// Make the equations unnumbered.
    #[must_use]
    pub fn unnumbered(mut self) -> Self {
        self.numbered = false;
        self
    }
}

/// Equation environment type (LaTeX-style).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EquationEnvironment {
    /// Align environment (aligned at &).
    Align,
    /// Gather environment (centered, no alignment).
    Gather,
    /// Multline environment (first line left, last right).
    Multline,
    /// Split environment (within equation).
    Split,
    /// Cases environment.
    Cases,
}

/// A single equation in a group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Equation {
    /// Optional unique identifier for referencing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Equation number (auto-generated or explicit).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,

    /// LaTeX content.
    pub latex: String,

    /// Optional label for referencing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

// ============================================================================
// Algorithm
// ============================================================================

/// A pseudocode algorithm block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Algorithm {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Algorithm name/title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Algorithm number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,

    /// Caption/description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,

    /// Input parameters.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<AlgorithmParam>,

    /// Output parameters.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<AlgorithmParam>,

    /// Algorithm body (pseudocode lines).
    pub body: Vec<AlgorithmLine>,

    /// Whether to show line numbers.
    #[serde(default = "default_true")]
    pub line_numbers: bool,
}

impl Algorithm {
    /// Create a new algorithm.
    #[must_use]
    pub fn new(body: Vec<AlgorithmLine>) -> Self {
        Self {
            id: None,
            name: None,
            number: None,
            caption: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            body,
            line_numbers: true,
        }
    }

    /// Set the name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Add an input parameter.
    #[must_use]
    pub fn with_input(mut self, name: impl Into<String>, description: impl Into<String>) -> Self {
        self.inputs.push(AlgorithmParam {
            name: name.into(),
            description: description.into(),
        });
        self
    }

    /// Add an output parameter.
    #[must_use]
    pub fn with_output(mut self, name: impl Into<String>, description: impl Into<String>) -> Self {
        self.outputs.push(AlgorithmParam {
            name: name.into(),
            description: description.into(),
        });
        self
    }
}

/// Algorithm input/output parameter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlgorithmParam {
    /// Parameter name.
    pub name: String,
    /// Parameter description.
    pub description: String,
}

/// A line of pseudocode in an algorithm.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlgorithmLine {
    /// Line number (auto-generated if not specified).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_number: Option<u32>,

    /// Indentation level.
    #[serde(default)]
    pub indent: u8,

    /// Line type.
    pub line_type: AlgorithmLineType,

    /// Line content.
    pub content: String,

    /// Optional comment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

/// Type of algorithm line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlgorithmLineType {
    /// Regular statement.
    Statement,
    /// If condition.
    If,
    /// Else if.
    ElseIf,
    /// Else.
    Else,
    /// End if.
    EndIf,
    /// For loop.
    For,
    /// End for.
    EndFor,
    /// While loop.
    While,
    /// End while.
    EndWhile,
    /// Function definition.
    Function,
    /// End function.
    EndFunction,
    /// Return statement.
    Return,
    /// Comment line.
    Comment,
}

// ============================================================================
// Numbering Configuration
// ============================================================================

/// Numbering configuration for academic content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NumberingConfig {
    /// Equation numbering configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equations: Option<NumberingStyle>,

    /// Theorem numbering configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theorems: Option<NumberingStyle>,

    /// Algorithm numbering configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub algorithms: Option<NumberingStyle>,

    /// Figure numbering configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub figures: Option<NumberingStyle>,

    /// Table numbering configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tables: Option<NumberingStyle>,
}

/// Numbering style configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NumberingStyle {
    /// Format string (e.g., "{chapter}.{number}").
    pub format: String,

    /// Whether to reset numbering per chapter.
    #[serde(default)]
    pub reset_per_chapter: bool,

    /// Starting number.
    #[serde(default = "default_start")]
    pub start: u32,
}

fn default_start() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theorem_new() {
        let thm = Theorem::new(TheoremVariant::Theorem, vec![]);
        assert_eq!(thm.variant, TheoremVariant::Theorem);
        assert!(thm.label.is_none());
        assert!(thm.number.is_none());
    }

    #[test]
    fn test_theorem_builder() {
        let thm = Theorem::new(TheoremVariant::Lemma, vec![])
            .with_label("Pumping Lemma")
            .with_number("4.2")
            .with_id("pumping")
            .with_attribution("Bar-Hillel et al.");

        assert_eq!(thm.variant, TheoremVariant::Lemma);
        assert_eq!(thm.label, Some("Pumping Lemma".to_string()));
        assert_eq!(thm.number, Some("4.2".to_string()));
        assert_eq!(thm.id, Some("pumping".to_string()));
        assert_eq!(thm.attribution, Some("Bar-Hillel et al.".to_string()));
    }

    #[test]
    fn test_theorem_variant_display() {
        assert_eq!(TheoremVariant::Theorem.to_string(), "Theorem");
        assert_eq!(TheoremVariant::Lemma.to_string(), "Lemma");
        assert_eq!(TheoremVariant::Corollary.to_string(), "Corollary");
    }

    #[test]
    fn test_theorem_serialization() {
        let thm = Theorem::new(TheoremVariant::Definition, vec![])
            .with_label("Continuity")
            .with_number("2.1");

        let json = serde_json::to_string(&thm).unwrap();
        assert!(json.contains("\"variant\":\"definition\""));
        assert!(json.contains("\"label\":\"Continuity\""));
        assert!(json.contains("\"number\":\"2.1\""));
    }

    #[test]
    fn test_proof_new() {
        let proof = Proof::new(vec![]);
        assert!(proof.theorem_ref.is_none());
        assert!(proof.method.is_none());
    }

    #[test]
    fn test_proof_builder() {
        let proof = Proof::new(vec![])
            .of_theorem("thm-pythagoras")
            .with_method(ProofMethod::Direct);

        assert_eq!(proof.theorem_ref, Some("thm-pythagoras".to_string()));
        assert_eq!(proof.method, Some(ProofMethod::Direct));
    }

    #[test]
    fn test_exercise_new() {
        let ex = Exercise::new(vec![]);
        assert!(ex.number.is_none());
        assert!(ex.difficulty.is_none());
    }

    #[test]
    fn test_exercise_builder() {
        let ex = Exercise::new(vec![])
            .with_number("3.5")
            .with_difficulty(Difficulty::Hard)
            .with_points(10);

        assert_eq!(ex.number, Some("3.5".to_string()));
        assert_eq!(ex.difficulty, Some(Difficulty::Hard));
        assert_eq!(ex.points, Some(10));
    }

    #[test]
    fn test_algorithm_new() {
        let alg = Algorithm::new(vec![]);
        assert!(alg.name.is_none());
        assert!(alg.line_numbers);
    }

    #[test]
    fn test_algorithm_builder() {
        let alg = Algorithm::new(vec![])
            .with_name("QuickSort")
            .with_input("A", "array to sort")
            .with_output("A", "sorted array");

        assert_eq!(alg.name, Some("QuickSort".to_string()));
        assert_eq!(alg.inputs.len(), 1);
        assert_eq!(alg.inputs[0].name, "A");
        assert_eq!(alg.outputs.len(), 1);
    }

    #[test]
    fn test_equation_group() {
        let eq = Equation {
            id: Some("eq1".to_string()),
            number: Some("(1)".to_string()),
            latex: "E = mc^2".to_string(),
            label: None,
        };
        let group = EquationGroup::new(EquationEnvironment::Align, vec![eq]);

        assert_eq!(group.environment, EquationEnvironment::Align);
        assert!(group.numbered);
        assert_eq!(group.equations.len(), 1);
    }

    #[test]
    fn test_abstract_new() {
        let abs = Abstract::new(vec![])
            .with_keywords(vec!["AI".to_string(), "Machine Learning".to_string()]);

        assert_eq!(abs.keywords.len(), 2);
        assert!(abs.sections.is_empty());
    }
}
