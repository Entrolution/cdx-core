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

    /// Content Anchor URIs of theorems this depends on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses: Option<Vec<String>>,

    /// Whether this restates an existing theorem.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restate: Option<bool>,
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
            uses: None,
            restate: None,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
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
    /// Structural induction.
    StructuralInduction,
    /// Counting argument.
    Counting,
    /// Probabilistic argument.
    Probabilistic,
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

    /// Equation lines in the group.
    pub lines: Vec<EquationLine>,
}

impl EquationGroup {
    /// Create a new equation group.
    #[must_use]
    pub fn new(environment: EquationEnvironment, lines: Vec<EquationLine>) -> Self {
        Self {
            id: None,
            environment,
            lines,
        }
    }

    /// Set the ID.
    #[must_use]
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
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
    /// Alignat environment (multiple alignment points).
    Alignat,
}

/// A single equation line in a group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EquationLine {
    /// Optional unique identifier for referencing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Equation content (LaTeX or other notation).
    pub value: String,

    /// Equation number (auto-generated or explicit).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,

    /// Custom tag instead of a number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

impl EquationLine {
    /// Create a new equation line.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            id: None,
            value: value.into(),
            number: None,
            tag: None,
        }
    }

    /// Set the equation number.
    #[must_use]
    pub fn with_number(mut self, number: impl Into<String>) -> Self {
        self.number = Some(number.into());
        self
    }

    /// Set a custom tag.
    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    /// Set the ID.
    #[must_use]
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

// ============================================================================
// Algorithm
// ============================================================================

fn default_true() -> bool {
    true
}

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

    /// Starting line number (default: 1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_line: Option<u32>,
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
            start_line: None,
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
// Cross-Reference Marks
// ============================================================================

/// An equation reference mark for cross-referencing equations.
///
/// Used inline to reference equations defined in equation groups.
///
/// # Example JSON
///
/// ```json
/// {
///   "type": "text",
///   "value": "(2.5)",
///   "marks": [
///     {
///       "type": "academic:equation-ref",
///       "target": "#eq-fx",
///       "format": "({number})"
///     }
///   ]
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EquationRef {
    /// Content Anchor URI to the equation (e.g., "#eq-fx").
    pub target: String,

    /// Display format with placeholder (default: "({number})").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

impl EquationRef {
    /// Create a new equation reference.
    #[must_use]
    pub fn new(target: impl Into<String>) -> Self {
        Self {
            target: target.into(),
            format: None,
        }
    }

    /// Set a custom format string.
    ///
    /// Use `{number}` as a placeholder for the equation number.
    #[must_use]
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Convert to an extension mark for use in text.
    #[must_use]
    pub fn to_extension_mark(&self) -> crate::content::ExtensionMark {
        let mut attrs = serde_json::json!({
            "target": self.target
        });
        if let Some(ref fmt) = self.format {
            attrs["format"] = serde_json::Value::String(fmt.clone());
        }
        crate::content::ExtensionMark::new("academic", "equation-ref").with_attributes(attrs)
    }
}

/// An algorithm reference mark for cross-referencing algorithms and their lines.
///
/// Used inline to reference algorithms or specific lines within algorithms.
///
/// # Example JSON
///
/// ```json
/// {
///   "type": "text",
///   "value": "Algorithm 1",
///   "marks": [
///     {
///       "type": "academic:algorithm-ref",
///       "target": "#alg-quicksort",
///       "format": "Algorithm {number}"
///     }
///   ]
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlgorithmRef {
    /// Content Anchor URI to the algorithm (e.g., "#alg-quicksort").
    pub target: String,

    /// Optional line label for line-specific references.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<String>,

    /// Display format with placeholders (e.g., "Algorithm {number}" or "line {line}").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

impl AlgorithmRef {
    /// Create a new algorithm reference.
    #[must_use]
    pub fn new(target: impl Into<String>) -> Self {
        Self {
            target: target.into(),
            line: None,
            format: None,
        }
    }

    /// Reference a specific line within the algorithm.
    #[must_use]
    pub fn with_line(mut self, line: impl Into<String>) -> Self {
        self.line = Some(line.into());
        self
    }

    /// Set a custom format string.
    ///
    /// Use `{number}` for algorithm number, `{line}` for line reference.
    #[must_use]
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Convert to an extension mark for use in text.
    #[must_use]
    pub fn to_extension_mark(&self) -> crate::content::ExtensionMark {
        let mut attrs = serde_json::json!({
            "target": self.target
        });
        if let Some(ref line) = self.line {
            attrs["line"] = serde_json::Value::String(line.clone());
        }
        if let Some(ref fmt) = self.format {
            attrs["format"] = serde_json::Value::String(fmt.clone());
        }
        crate::content::ExtensionMark::new("academic", "algorithm-ref").with_attributes(attrs)
    }
}

/// A theorem reference mark for cross-referencing theorems, lemmas, etc.
///
/// # Example JSON
///
/// ```json
/// {
///   "type": "text",
///   "value": "Theorem 3.1",
///   "marks": [
///     {
///       "type": "academic:theorem-ref",
///       "target": "#thm-pythagoras",
///       "format": "{variant} {number}"
///     }
///   ]
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TheoremRef {
    /// Content Anchor URI to the theorem (e.g., "#thm-pythagoras").
    pub target: String,

    /// Display format with placeholders.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

impl TheoremRef {
    /// Create a new theorem reference.
    #[must_use]
    pub fn new(target: impl Into<String>) -> Self {
        Self {
            target: target.into(),
            format: None,
        }
    }

    /// Set a custom format string.
    ///
    /// Use `{variant}` for theorem type, `{number}` for theorem number.
    #[must_use]
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Convert to an extension mark for use in text.
    #[must_use]
    pub fn to_extension_mark(&self) -> crate::content::ExtensionMark {
        let mut attrs = serde_json::json!({
            "target": self.target
        });
        if let Some(ref fmt) = self.format {
            attrs["format"] = serde_json::Value::String(fmt.clone());
        }
        crate::content::ExtensionMark::new("academic", "theorem-ref").with_attributes(attrs)
    }
}

// ============================================================================
// Numbering Configuration
// ============================================================================

/// When to reset counters. Uses heading level identifiers
/// corresponding to the core heading block's level attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ResetTrigger {
    /// Reset at heading level 1.
    Heading1,
    /// Reset at heading level 2.
    Heading2,
    /// Reset at heading level 3.
    Heading3,
    /// Reset at heading level 4.
    Heading4,
    /// Reset at heading level 5.
    Heading5,
    /// Reset at heading level 6.
    Heading6,
    /// Never reset.
    None,
}

/// Numbering style pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumberingStylePattern {
    /// Sequential numbering (e.g., 1, 2, 3).
    #[serde(rename = "number")]
    Number,
    /// Chapter-scoped numbering (e.g., 2.1, 2.2).
    #[serde(rename = "chapter.number")]
    ChapterNumber,
    /// Section-scoped numbering (e.g., 3.1, 3.2).
    #[serde(rename = "section.number")]
    SectionNumber,
    /// Chapter-and-section-scoped numbering (e.g., 2.3.1).
    #[serde(rename = "chapter.section.number")]
    ChapterSectionNumber,
}

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
    /// Numbering style pattern.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "format")]
    pub style: Option<NumberingStylePattern>,

    /// When to reset counters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reset_on: Option<ResetTrigger>,

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
        let line = EquationLine::new("E = mc^2")
            .with_id("eq1")
            .with_number("(1)");
        let group = EquationGroup::new(EquationEnvironment::Align, vec![line]);

        assert_eq!(group.environment, EquationEnvironment::Align);
        assert_eq!(group.lines.len(), 1);
        assert_eq!(group.lines[0].value, "E = mc^2");
        assert_eq!(group.lines[0].id, Some("eq1".to_string()));
        assert_eq!(group.lines[0].number, Some("(1)".to_string()));
    }

    #[test]
    fn test_equation_line_with_tag() {
        let line = EquationLine::new("a^2 + b^2 = c^2").with_tag("*");
        assert_eq!(line.tag, Some("*".to_string()));
        assert!(line.number.is_none());
    }

    #[test]
    fn test_equation_line_serde_roundtrip() {
        let line = EquationLine::new("f(x) = ax + b")
            .with_id("eq-fx")
            .with_number("2.1")
            .with_tag("linear");
        let json = serde_json::to_string(&line).unwrap();
        assert!(json.contains("\"value\":\"f(x) = ax + b\""));
        assert!(json.contains("\"number\":\"2.1\""));
        assert!(json.contains("\"tag\":\"linear\""));

        let parsed: EquationLine = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.value, "f(x) = ax + b");
        assert_eq!(parsed.number, Some("2.1".to_string()));
        assert_eq!(parsed.tag, Some("linear".to_string()));
    }

    #[test]
    fn test_equation_line_without_tag_defaults_to_none() {
        let json = r#"{"value": "x + y"}"#;
        let line: EquationLine = serde_json::from_str(json).unwrap();
        assert!(line.tag.is_none());
        assert!(line.number.is_none());
        assert!(line.id.is_none());
    }

    #[test]
    fn test_equation_group_with_lines_serde() {
        let group = EquationGroup::new(
            EquationEnvironment::Gather,
            vec![
                EquationLine::new("a = b").with_number("1"),
                EquationLine::new("c = d").with_number("2"),
            ],
        )
        .with_id("eq-group-1");

        let json = serde_json::to_string(&group).unwrap();
        assert!(json.contains("\"lines\""));
        assert!(json.contains("\"environment\":\"gather\""));

        let parsed: EquationGroup = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.lines.len(), 2);
        assert_eq!(parsed.id, Some("eq-group-1".to_string()));
    }

    #[test]
    fn test_alignat_environment_serialization() {
        let group = EquationGroup::new(
            EquationEnvironment::Alignat,
            vec![EquationLine::new("x &= y &= z")],
        );
        let json = serde_json::to_string(&group).unwrap();
        assert!(json.contains("\"environment\":\"alignat\""));

        let parsed: EquationGroup = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.environment, EquationEnvironment::Alignat);
    }

    #[test]
    fn test_abstract_new() {
        let abs = Abstract::new(vec![])
            .with_keywords(vec!["AI".to_string(), "Machine Learning".to_string()]);

        assert_eq!(abs.keywords.len(), 2);
        assert!(abs.sections.is_empty());
    }

    #[test]
    fn test_equation_ref() {
        let eq_ref = EquationRef::new("#eq-pythagoras");
        assert_eq!(eq_ref.target, "#eq-pythagoras");
        assert!(eq_ref.format.is_none());

        let eq_ref_fmt = eq_ref.with_format("Equation ({number})");
        assert_eq!(eq_ref_fmt.format, Some("Equation ({number})".to_string()));
    }

    #[test]
    fn test_equation_ref_to_mark() {
        let eq_ref = EquationRef::new("#eq-1").with_format("({number})");
        let mark = eq_ref.to_extension_mark();

        assert_eq!(mark.namespace, "academic");
        assert_eq!(mark.mark_type, "equation-ref");
        assert_eq!(mark.get_string_attribute("target"), Some("#eq-1"));
        assert_eq!(mark.get_string_attribute("format"), Some("({number})"));
    }

    #[test]
    fn test_algorithm_ref() {
        let alg_ref = AlgorithmRef::new("#alg-quicksort");
        assert_eq!(alg_ref.target, "#alg-quicksort");
        assert!(alg_ref.line.is_none());
        assert!(alg_ref.format.is_none());
    }

    #[test]
    fn test_algorithm_ref_with_line() {
        let alg_ref = AlgorithmRef::new("#alg-bisection")
            .with_line("loop")
            .with_format("line {line}");

        assert_eq!(alg_ref.target, "#alg-bisection");
        assert_eq!(alg_ref.line, Some("loop".to_string()));
        assert_eq!(alg_ref.format, Some("line {line}".to_string()));
    }

    #[test]
    fn test_algorithm_ref_to_mark() {
        let alg_ref = AlgorithmRef::new("#alg-1")
            .with_line("start")
            .with_format("Algorithm {number}, line {line}");
        let mark = alg_ref.to_extension_mark();

        assert_eq!(mark.namespace, "academic");
        assert_eq!(mark.mark_type, "algorithm-ref");
        assert_eq!(mark.get_string_attribute("target"), Some("#alg-1"));
        assert_eq!(mark.get_string_attribute("line"), Some("start"));
        assert_eq!(
            mark.get_string_attribute("format"),
            Some("Algorithm {number}, line {line}")
        );
    }

    #[test]
    fn test_theorem_ref() {
        let thm_ref = TheoremRef::new("#thm-pythagoras");
        assert_eq!(thm_ref.target, "#thm-pythagoras");
        assert!(thm_ref.format.is_none());
    }

    #[test]
    fn test_theorem_ref_to_mark() {
        let thm_ref = TheoremRef::new("#thm-1").with_format("{variant} {number}");
        let mark = thm_ref.to_extension_mark();

        assert_eq!(mark.namespace, "academic");
        assert_eq!(mark.mark_type, "theorem-ref");
        assert_eq!(mark.get_string_attribute("target"), Some("#thm-1"));
        assert_eq!(
            mark.get_string_attribute("format"),
            Some("{variant} {number}")
        );
    }

    #[test]
    fn test_equation_ref_serialization() {
        let eq_ref = EquationRef::new("#eq-fx").with_format("({number})");
        let json = serde_json::to_string(&eq_ref).unwrap();
        assert!(json.contains("\"target\":\"#eq-fx\""));
        assert!(json.contains("\"format\":\"({number})\""));

        // Deserialize back
        let parsed: EquationRef = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.target, "#eq-fx");
        assert_eq!(parsed.format, Some("({number})".to_string()));
    }

    #[test]
    fn test_algorithm_ref_serialization() {
        let alg_ref = AlgorithmRef::new("#alg-sort")
            .with_line("pivot")
            .with_format("line {line}");
        let json = serde_json::to_string(&alg_ref).unwrap();
        assert!(json.contains("\"target\":\"#alg-sort\""));
        assert!(json.contains("\"line\":\"pivot\""));
        assert!(json.contains("\"format\":\"line {line}\""));
    }

    #[test]
    fn test_theorem_uses_and_restate_roundtrip() {
        let thm = Theorem {
            id: Some("thm-2".to_string()),
            variant: TheoremVariant::Corollary,
            label: None,
            number: None,
            children: vec![],
            attribution: None,
            citation: None,
            uses: Some(vec!["#thm-1".to_string(), "#lemma-1".to_string()]),
            restate: Some(true),
        };
        let json = serde_json::to_string(&thm).unwrap();
        assert!(json.contains("\"uses\":[\"#thm-1\",\"#lemma-1\"]"));
        assert!(json.contains("\"restate\":true"));

        let parsed: Theorem = serde_json::from_str(&json).unwrap();
        assert_eq!(
            parsed.uses,
            Some(vec!["#thm-1".to_string(), "#lemma-1".to_string()])
        );
        assert_eq!(parsed.restate, Some(true));
    }

    #[test]
    fn test_theorem_without_new_fields_defaults_to_none() {
        let json = r#"{
            "variant": "theorem",
            "children": []
        }"#;
        let thm: Theorem = serde_json::from_str(json).unwrap();
        assert!(thm.uses.is_none());
        assert!(thm.restate.is_none());
    }

    #[test]
    fn test_new_proof_method_variants() {
        let methods = [
            (ProofMethod::StructuralInduction, "structuralinduction"),
            (ProofMethod::Counting, "counting"),
            (ProofMethod::Probabilistic, "probabilistic"),
        ];
        for (method, expected_str) in methods {
            let json = serde_json::to_string(&method).unwrap();
            assert_eq!(json, format!("\"{expected_str}\""));
            let parsed: ProofMethod = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, method);
        }
    }

    #[test]
    fn test_algorithm_start_line_roundtrip() {
        let alg = Algorithm {
            id: None,
            name: Some("BFS".to_string()),
            number: None,
            caption: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            body: vec![],
            line_numbers: true,
            start_line: Some(10),
        };
        let json = serde_json::to_string(&alg).unwrap();
        assert!(json.contains("\"startLine\":10"));

        let parsed: Algorithm = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.start_line, Some(10));
    }

    #[test]
    fn test_algorithm_without_start_line_defaults_to_none() {
        let json = r#"{
            "body": [],
            "lineNumbers": true
        }"#;
        let alg: Algorithm = serde_json::from_str(json).unwrap();
        assert!(alg.start_line.is_none());
    }
}
