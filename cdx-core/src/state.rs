//! Document state machine.
//!
//! Codex documents progress through a defined lifecycle:
//!
//! ```text
//! draft → review → frozen → published
//! ```
//!
//! Each state has specific characteristics and requirements.

use serde::{Deserialize, Serialize};

/// Document lifecycle state.
///
/// Documents progress through states that determine what operations are allowed
/// and what integrity guarantees are enforced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocumentState {
    /// Active editing state.
    ///
    /// - Content can be freely modified
    /// - Document ID may be "pending"
    /// - No signatures required
    Draft,

    /// Review/approval state.
    ///
    /// - Content can still be modified
    /// - Document ID should be computed
    /// - Signatures optional
    Review,

    /// Immutable state.
    ///
    /// - Content cannot be modified
    /// - Document ID must be computed and valid
    /// - At least one signature required
    /// - Lineage required (parent required only if forked)
    Frozen,

    /// Distribution state.
    ///
    /// - Same immutability as frozen
    /// - Indicates document is ready for public distribution
    /// - At least one signature required
    /// - Lineage required (parent required only if forked)
    Published,
}

impl DocumentState {
    /// Returns whether the document content can be modified in this state.
    #[must_use]
    pub const fn is_editable(&self) -> bool {
        matches!(self, Self::Draft | Self::Review)
    }

    /// Returns whether the document is immutable in this state.
    #[must_use]
    pub const fn is_immutable(&self) -> bool {
        matches!(self, Self::Frozen | Self::Published)
    }

    /// Returns whether signatures are required in this state.
    #[must_use]
    pub const fn requires_signature(&self) -> bool {
        matches!(self, Self::Frozen | Self::Published)
    }

    /// Returns whether lineage is required in this state.
    ///
    /// Note: The `lineage.parent` field is only required for forked documents.
    /// Root documents (first version) can have lineage with `parent: None`.
    #[must_use]
    pub const fn requires_lineage(&self) -> bool {
        matches!(self, Self::Frozen | Self::Published)
    }

    /// Returns whether the document ID must be computed (not "pending").
    #[must_use]
    pub const fn requires_computed_id(&self) -> bool {
        matches!(self, Self::Review | Self::Frozen | Self::Published)
    }

    /// Returns whether a precise layout is required in this state.
    ///
    /// Frozen and published documents require at least one precise layout
    /// to ensure exact visual reproduction as part of the immutable record.
    #[must_use]
    pub const fn requires_precise_layout(&self) -> bool {
        matches!(self, Self::Frozen | Self::Published)
    }

    /// Check if transitioning to the target state is valid.
    #[must_use]
    pub const fn can_transition_to(&self, target: Self) -> bool {
        use DocumentState::{Draft, Frozen, Published, Review};
        matches!(
            (self, target),
            (Draft, Review) | (Review, Frozen | Draft) | (Frozen, Published)
        )
    }

    /// Get all valid target states from the current state.
    #[must_use]
    pub const fn valid_transitions(&self) -> &'static [DocumentState] {
        use DocumentState::{Draft, Frozen, Published, Review};
        match self {
            Draft => &[Review],
            Review => &[Draft, Frozen],
            Frozen => &[Published],
            Published => &[],
        }
    }
}

impl std::fmt::Display for DocumentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Review => write!(f, "review"),
            Self::Frozen => write!(f, "frozen"),
            Self::Published => write!(f, "published"),
        }
    }
}

impl std::str::FromStr for DocumentState {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(Self::Draft),
            "review" => Ok(Self::Review),
            "frozen" => Ok(Self::Frozen),
            "published" => Ok(Self::Published),
            _ => Err(crate::Error::InvalidManifest {
                reason: format!("invalid state: {s}"),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editability() {
        assert!(DocumentState::Draft.is_editable());
        assert!(DocumentState::Review.is_editable());
        assert!(!DocumentState::Frozen.is_editable());
        assert!(!DocumentState::Published.is_editable());
    }

    #[test]
    fn test_immutability() {
        assert!(!DocumentState::Draft.is_immutable());
        assert!(!DocumentState::Review.is_immutable());
        assert!(DocumentState::Frozen.is_immutable());
        assert!(DocumentState::Published.is_immutable());
    }

    #[test]
    fn test_valid_transitions() {
        use DocumentState::{Draft, Frozen, Published, Review};

        // Draft can go to Review
        assert!(Draft.can_transition_to(Review));
        assert!(!Draft.can_transition_to(Frozen));
        assert!(!Draft.can_transition_to(Published));

        // Review can go to Draft or Frozen
        assert!(Review.can_transition_to(Draft));
        assert!(Review.can_transition_to(Frozen));
        assert!(!Review.can_transition_to(Published));

        // Frozen can go to Published
        assert!(!Frozen.can_transition_to(Draft));
        assert!(!Frozen.can_transition_to(Review));
        assert!(Frozen.can_transition_to(Published));

        // Published is terminal
        assert!(!Published.can_transition_to(Draft));
        assert!(!Published.can_transition_to(Review));
        assert!(!Published.can_transition_to(Frozen));
    }

    #[test]
    fn test_serialization() {
        let state = DocumentState::Frozen;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"frozen\"");

        let parsed: DocumentState = serde_json::from_str("\"review\"").unwrap();
        assert_eq!(parsed, DocumentState::Review);
    }

    #[test]
    fn test_display() {
        assert_eq!(DocumentState::Draft.to_string(), "draft");
        assert_eq!(DocumentState::Published.to_string(), "published");
    }

    #[test]
    fn test_from_str() {
        assert_eq!(
            "draft".parse::<DocumentState>().unwrap(),
            DocumentState::Draft
        );
        assert_eq!(
            "FROZEN".parse::<DocumentState>().unwrap(),
            DocumentState::Frozen
        );
        assert!("invalid".parse::<DocumentState>().is_err());
    }

    #[test]
    fn test_precise_layout_requirement() {
        assert!(!DocumentState::Draft.requires_precise_layout());
        assert!(!DocumentState::Review.requires_precise_layout());
        assert!(DocumentState::Frozen.requires_precise_layout());
        assert!(DocumentState::Published.requires_precise_layout());
    }
}
