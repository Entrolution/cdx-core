//! Access control and permissions for Codex documents.
//!
//! This module provides a permission system for controlling access to document
//! operations like viewing, printing, copying, editing, and annotating.

use serde::{Deserialize, Serialize};

/// Access control settings for a document.
///
/// Defines default permissions and per-principal permission grants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessControl {
    /// Default permissions for unauthenticated/unknown principals.
    pub default: Permissions,

    /// Per-principal permission grants.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub grants: Vec<PermissionGrant>,
}

impl AccessControl {
    /// Create access control with default permissions.
    #[must_use]
    pub fn new(default: Permissions) -> Self {
        Self {
            default,
            grants: Vec::new(),
        }
    }

    /// Create access control with all permissions denied by default.
    #[must_use]
    pub fn deny_all() -> Self {
        Self::new(Permissions::none())
    }

    /// Create access control with read-only permissions by default.
    #[must_use]
    pub fn read_only() -> Self {
        Self::new(Permissions::read_only())
    }

    /// Create access control with all permissions granted by default.
    #[must_use]
    pub fn full_access() -> Self {
        Self::new(Permissions::all())
    }

    /// Add a permission grant for a principal.
    #[must_use]
    pub fn with_grant(mut self, grant: PermissionGrant) -> Self {
        self.grants.push(grant);
        self
    }

    /// Get effective permissions for a principal.
    ///
    /// Searches for matching grants and returns the most specific permissions,
    /// or the default permissions if no grants match.
    #[must_use]
    pub fn permissions_for(&self, principal: &Principal) -> &Permissions {
        // Find the most specific matching grant
        for grant in &self.grants {
            if grant.principal.matches(principal) {
                return &grant.permissions;
            }
        }
        &self.default
    }

    /// Check if a principal can perform an operation.
    #[must_use]
    pub fn can(&self, principal: &Principal, operation: Operation) -> bool {
        let perms = self.permissions_for(principal);
        match operation {
            Operation::View => perms.view,
            Operation::Print => perms.print,
            Operation::Copy => perms.copy,
            Operation::Annotate => perms.annotate,
            Operation::Edit => perms.edit,
            Operation::Sign => perms.sign,
            Operation::Decrypt => perms.decrypt,
        }
    }
}

impl Default for AccessControl {
    fn default() -> Self {
        Self::read_only()
    }
}

/// Permission settings.
///
/// This struct uses boolean flags for each permission type as specified by the
/// Codex Document Format specification. Each flag represents an independent
/// permission that can be granted or denied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)]
pub struct Permissions {
    /// Permission to view/read the document.
    #[serde(default = "default_true")]
    pub view: bool,

    /// Permission to print the document.
    #[serde(default)]
    pub print: bool,

    /// Permission to copy content from the document.
    #[serde(default)]
    pub copy: bool,

    /// Permission to add annotations/comments.
    #[serde(default)]
    pub annotate: bool,

    /// Permission to edit the document.
    #[serde(default)]
    pub edit: bool,

    /// Permission to sign the document.
    #[serde(default)]
    pub sign: bool,

    /// Permission to decrypt (if encrypted).
    #[serde(default)]
    pub decrypt: bool,
}

fn default_true() -> bool {
    true
}

impl Permissions {
    /// Create permissions with all operations denied.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            view: false,
            print: false,
            copy: false,
            annotate: false,
            edit: false,
            sign: false,
            decrypt: false,
        }
    }

    /// Create permissions with all operations allowed.
    #[must_use]
    pub const fn all() -> Self {
        Self {
            view: true,
            print: true,
            copy: true,
            annotate: true,
            edit: true,
            sign: true,
            decrypt: true,
        }
    }

    /// Create read-only permissions (view only).
    #[must_use]
    pub const fn read_only() -> Self {
        Self {
            view: true,
            print: false,
            copy: false,
            annotate: false,
            edit: false,
            sign: false,
            decrypt: false,
        }
    }

    /// Create view and print permissions.
    #[must_use]
    pub const fn view_and_print() -> Self {
        Self {
            view: true,
            print: true,
            copy: false,
            annotate: false,
            edit: false,
            sign: false,
            decrypt: false,
        }
    }

    /// Create reviewer permissions (view, annotate, sign).
    #[must_use]
    pub const fn reviewer() -> Self {
        Self {
            view: true,
            print: true,
            copy: true,
            annotate: true,
            edit: false,
            sign: true,
            decrypt: true,
        }
    }

    /// Create editor permissions (all except sign).
    #[must_use]
    pub const fn editor() -> Self {
        Self {
            view: true,
            print: true,
            copy: true,
            annotate: true,
            edit: true,
            sign: false,
            decrypt: true,
        }
    }
}

impl Default for Permissions {
    fn default() -> Self {
        Self::read_only()
    }
}

/// A permission grant for a specific principal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionGrant {
    /// The principal receiving the permissions.
    pub principal: Principal,

    /// The granted permissions.
    pub permissions: Permissions,
}

impl PermissionGrant {
    /// Create a new permission grant.
    #[must_use]
    pub fn new(principal: Principal, permissions: Permissions) -> Self {
        Self {
            principal,
            permissions,
        }
    }

    /// Grant full access to a user.
    #[must_use]
    pub fn full_access_for_user(email: impl Into<String>) -> Self {
        Self::new(Principal::User(email.into()), Permissions::all())
    }

    /// Grant reviewer access to a user.
    #[must_use]
    pub fn reviewer_for_user(email: impl Into<String>) -> Self {
        Self::new(Principal::User(email.into()), Permissions::reviewer())
    }

    /// Grant editor access to a user.
    #[must_use]
    pub fn editor_for_user(email: impl Into<String>) -> Self {
        Self::new(Principal::User(email.into()), Permissions::editor())
    }
}

/// A security principal (user, group, or role).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "id", rename_all = "lowercase")]
pub enum Principal {
    /// A specific user identified by email or ID.
    User(String),

    /// A group of users.
    Group(String),

    /// A role (e.g., "admin", "reviewer").
    Role(String),

    /// Everyone (wildcard).
    Everyone,
}

impl Principal {
    /// Create a user principal.
    #[must_use]
    pub fn user(id: impl Into<String>) -> Self {
        Self::User(id.into())
    }

    /// Create a group principal.
    #[must_use]
    pub fn group(id: impl Into<String>) -> Self {
        Self::Group(id.into())
    }

    /// Create a role principal.
    #[must_use]
    pub fn role(id: impl Into<String>) -> Self {
        Self::Role(id.into())
    }

    /// Check if this principal matches another.
    ///
    /// `Everyone` matches all principals. Otherwise, exact match is required.
    #[must_use]
    pub fn matches(&self, other: &Principal) -> bool {
        match self {
            Self::Everyone => true,
            Self::User(id) => matches!(other, Self::User(other_id) if id == other_id),
            Self::Group(id) => matches!(other, Self::Group(other_id) if id == other_id),
            Self::Role(id) => matches!(other, Self::Role(other_id) if id == other_id),
        }
    }
}

impl std::fmt::Display for Principal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User(id) => write!(f, "user:{id}"),
            Self::Group(id) => write!(f, "group:{id}"),
            Self::Role(id) => write!(f, "role:{id}"),
            Self::Everyone => write!(f, "*"),
        }
    }
}

/// Operations that can be controlled by permissions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Operation {
    /// View/read the document.
    View,
    /// Print the document.
    Print,
    /// Copy content from the document.
    Copy,
    /// Add annotations or comments.
    Annotate,
    /// Edit the document content.
    Edit,
    /// Add a signature.
    Sign,
    /// Decrypt the document (if encrypted).
    Decrypt,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permissions_none() {
        let perms = Permissions::none();
        assert!(!perms.view);
        assert!(!perms.print);
        assert!(!perms.edit);
    }

    #[test]
    fn test_permissions_all() {
        let perms = Permissions::all();
        assert!(perms.view);
        assert!(perms.print);
        assert!(perms.copy);
        assert!(perms.annotate);
        assert!(perms.edit);
        assert!(perms.sign);
        assert!(perms.decrypt);
    }

    #[test]
    fn test_permissions_read_only() {
        let perms = Permissions::read_only();
        assert!(perms.view);
        assert!(!perms.print);
        assert!(!perms.edit);
    }

    #[test]
    fn test_access_control_default() {
        let ac = AccessControl::read_only();
        let user = Principal::user("test@example.com");

        assert!(ac.can(&user, Operation::View));
        assert!(!ac.can(&user, Operation::Print));
        assert!(!ac.can(&user, Operation::Edit));
    }

    #[test]
    fn test_access_control_with_grant() {
        let ac = AccessControl::deny_all()
            .with_grant(PermissionGrant::full_access_for_user("admin@example.com"));

        let admin = Principal::user("admin@example.com");
        let user = Principal::user("user@example.com");

        assert!(ac.can(&admin, Operation::View));
        assert!(ac.can(&admin, Operation::Edit));
        assert!(!ac.can(&user, Operation::View));
    }

    #[test]
    fn test_principal_matches() {
        let everyone = Principal::Everyone;
        let user = Principal::user("test@example.com");

        assert!(everyone.matches(&user));
        assert!(user.matches(&user));
        assert!(!user.matches(&Principal::user("other@example.com")));
    }

    #[test]
    fn test_principal_display() {
        assert_eq!(
            Principal::user("test@example.com").to_string(),
            "user:test@example.com"
        );
        assert_eq!(Principal::group("admins").to_string(), "group:admins");
        assert_eq!(Principal::role("reviewer").to_string(), "role:reviewer");
        assert_eq!(Principal::Everyone.to_string(), "*");
    }

    #[test]
    fn test_serialization() {
        let ac = AccessControl::read_only().with_grant(PermissionGrant::new(
            Principal::user("admin@example.com"),
            Permissions::all(),
        ));

        let json = serde_json::to_string_pretty(&ac).unwrap();
        assert!(json.contains("\"view\": true"));
        assert!(json.contains("admin@example.com"));

        let deserialized: AccessControl = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.grants.len(), 1);
    }

    #[test]
    fn test_reviewer_permissions() {
        let perms = Permissions::reviewer();
        assert!(perms.view);
        assert!(perms.annotate);
        assert!(perms.sign);
        assert!(!perms.edit);
    }

    #[test]
    fn test_editor_permissions() {
        let perms = Permissions::editor();
        assert!(perms.view);
        assert!(perms.edit);
        assert!(!perms.sign);
    }
}
