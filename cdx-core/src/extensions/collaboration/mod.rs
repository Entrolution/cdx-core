//! Collaboration extension for comments, annotations, and change tracking.
//!
//! This extension provides collaborative editing features for CDX documents.
//!
//! # Features
//!
//! - **Comments**: Inline comments with threading and replies
//! - **Highlights**: Text highlighting with colors
//! - **Suggestions**: Proposed text changes (accept/reject workflow)
//! - **Reactions**: Emoji reactions to content
//! - **Change Tracking**: Track insertions, deletions, and modifications
//!
//! # Example
//!
//! ```json
//! {
//!   "type": "collaboration:comment",
//!   "id": "comment-1",
//!   "blockRef": "block-42",
//!   "range": {"start": 10, "end": 25},
//!   "author": {"name": "Alice", "email": "alice@example.com"},
//!   "content": "Consider rephrasing this section.",
//!   "created": "2024-01-15T10:30:00Z"
//! }
//! ```

mod change_tracking;
mod comment;
mod crdt;
mod revision;
mod session;
mod thread;

// Re-export all public types for backwards compatibility
pub use change_tracking::{ChangeStatus, ChangeTracking, ChangeType, TrackedChange};
pub use comment::{
    Collaborator, Comment, CommentType, HighlightColor, Priority, SuggestionStatus, TextRange,
};
pub use crdt::{CrdtFormat, CrdtMetadata, Peer, SyncState, TextCrdtMetadata, TextCrdtPosition};
pub use revision::{MaterializationEvent, MaterializationReason, Revision, RevisionHistory};
pub use session::{CollaborationSession, CursorPosition, Participant, Selection, SessionStatus};
pub use thread::CommentThread;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_new() {
        let author = Collaborator::new("Alice");
        let comment = Comment::new("c1", "block-1", author, "Great point!");

        assert_eq!(comment.id, "c1");
        assert_eq!(comment.block_ref, "block-1");
        assert_eq!(comment.content, "Great point!");
        assert!(!comment.resolved);
    }

    #[test]
    fn test_comment_highlight() {
        let author = Collaborator::new("Bob");
        let range = TextRange::new(10, 20);
        let comment = Comment::highlight("h1", "block-2", range, author, HighlightColor::Yellow);

        assert!(matches!(
            comment.comment_type,
            CommentType::Highlight {
                color: HighlightColor::Yellow
            }
        ));
        assert_eq!(comment.range, Some(TextRange::new(10, 20)));
    }

    #[test]
    fn test_comment_suggestion() {
        let author = Collaborator::new("Carol");
        let range = TextRange::new(0, 5);
        let comment = Comment::suggestion("s1", "block-3", range, author, "Hello", "Hi");

        assert!(comment.is_suggestion());
        assert_eq!(comment.suggestion_status(), Some(SuggestionStatus::Pending));
    }

    #[test]
    fn test_suggestion_accept_reject() {
        let author = Collaborator::new("Dave");
        let range = TextRange::new(0, 5);
        let mut comment = Comment::suggestion("s1", "block-3", range, author, "old", "new");

        assert!(comment.accept_suggestion());
        assert_eq!(
            comment.suggestion_status(),
            Some(SuggestionStatus::Accepted)
        );

        let author2 = Collaborator::new("Eve");
        let range2 = TextRange::new(0, 5);
        let mut comment2 = Comment::suggestion("s2", "block-4", range2, author2, "old", "new");

        assert!(comment2.reject_suggestion());
        assert_eq!(
            comment2.suggestion_status(),
            Some(SuggestionStatus::Rejected)
        );
    }

    #[test]
    fn test_comment_resolve() {
        let author = Collaborator::new("Frank");
        let resolver = Collaborator::new("Grace");
        let mut comment = Comment::new("c1", "block-1", author, "Fix this");

        comment.resolve(resolver.clone());

        assert!(comment.resolved);
        assert_eq!(comment.resolved_by.as_ref().unwrap().name, "Grace");
        assert!(comment.resolved_at.is_some());

        comment.unresolve();
        assert!(!comment.resolved);
    }

    #[test]
    fn test_comment_reply() {
        let author1 = Collaborator::new("Alice");
        let author2 = Collaborator::new("Bob");
        let mut comment = Comment::new("c1", "block-1", author1, "Question?");
        let reply = Comment::new("c2", "block-1", author2, "Answer!");

        comment.add_reply(reply);

        assert_eq!(comment.replies.len(), 1);
        assert_eq!(comment.replies[0].parent_id, Some("c1".to_string()));
    }

    #[test]
    fn test_text_range() {
        let range = TextRange::new(10, 20);
        assert_eq!(range.len(), 10);
        assert!(!range.is_empty());
        assert!(range.contains(15));
        assert!(!range.contains(25));

        let other = TextRange::new(15, 25);
        assert!(range.overlaps(&other));

        let contained = TextRange::new(12, 18);
        assert!(range.contains_range(&contained));
    }

    #[test]
    fn test_author_builder() {
        let author = Collaborator::new("Alice")
            .with_email("alice@example.com")
            .with_user_id("user-123");

        assert_eq!(author.name, "Alice");
        assert_eq!(author.email, Some("alice@example.com".to_string()));
        assert_eq!(author.user_id, Some("user-123".to_string()));
    }

    #[test]
    fn test_change_tracking() {
        let base = "sha256:abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234"
            .parse()
            .unwrap();
        let mut tracking = ChangeTracking::new(base);
        let author = Collaborator::new("Alice");

        let change = TrackedChange::new("ch1", ChangeType::Insert, "block-1", author);
        tracking.add_change(change);

        assert_eq!(tracking.len(), 1);
        assert!(!tracking.is_empty());
        assert_eq!(tracking.pending_changes().len(), 1);
    }

    #[test]
    fn test_change_accept_reject() {
        let base = "sha256:abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234"
            .parse()
            .unwrap();
        let mut tracking = ChangeTracking::new(base);
        let author = Collaborator::new("Alice");

        tracking.add_change(TrackedChange::new(
            "ch1",
            ChangeType::Insert,
            "block-1",
            author.clone(),
        ));
        tracking.add_change(TrackedChange::new(
            "ch2",
            ChangeType::Delete,
            "block-2",
            author,
        ));

        assert!(tracking.accept_change("ch1"));
        assert!(tracking.reject_change("ch2"));

        assert_eq!(tracking.pending_changes().len(), 0);
        assert_eq!(tracking.changes[0].status, ChangeStatus::Accepted);
        assert_eq!(tracking.changes[1].status, ChangeStatus::Rejected);
    }

    #[test]
    fn test_tracked_change_inline() {
        let author = Collaborator::new("Bob");
        let range = TextRange::new(10, 20);
        let change =
            TrackedChange::inline_text("ch1", "block-1", author, range, "original", "replacement");

        assert_eq!(change.range, Some(TextRange::new(10, 20)));
        assert_eq!(change.original_text, Some("original".to_string()));
        assert_eq!(change.new_text, Some("replacement".to_string()));
    }

    #[test]
    fn test_comment_thread() {
        let mut thread = CommentThread::new();
        let author = Collaborator::new("Alice");

        thread.add(Comment::new("c1", "block-1", author.clone(), "First"));
        thread.add(Comment::new("c2", "block-2", author, "Second"));

        assert_eq!(thread.len(), 2);
        assert!(thread.get("c1").is_some());
        assert_eq!(thread.for_block("block-1").len(), 1);
    }

    #[test]
    fn test_collaboration_session() {
        let doc_id = "sha256:abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234"
            .parse()
            .unwrap();
        let mut session = CollaborationSession::new("session-1", doc_id);

        let participant = Participant::new(Collaborator::new("Alice").with_user_id("user-1"));
        session.add_participant(participant);

        assert_eq!(session.participant_count(), 1);
        assert_eq!(session.status, SessionStatus::Active);

        session.end();
        assert_eq!(session.status, SessionStatus::Ended);
    }

    #[test]
    fn test_cursor_and_selection() {
        let cursor = CursorPosition::new("block-1", 42);
        assert_eq!(cursor.block_ref, "block-1");
        assert_eq!(cursor.offset, 42);

        let selection = Selection::within_block("block-2", 10, 20);
        assert_eq!(selection.start.block_ref, "block-2");
        assert_eq!(selection.start.offset, 10);
        assert_eq!(selection.end.offset, 20);
    }

    #[test]
    fn test_comment_serialization() {
        let author = Collaborator::new("Alice").with_email("alice@example.com");
        let comment = Comment::new("c1", "block-1", author, "Test comment");

        let json = serde_json::to_string(&comment).unwrap();
        assert!(json.contains("\"id\":\"c1\""));
        assert!(json.contains("\"blockRef\":\"block-1\""));

        let deserialized: Comment = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "c1");
    }

    #[test]
    fn test_change_tracking_serialization() {
        let base = "sha256:abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234"
            .parse()
            .unwrap();
        let tracking = ChangeTracking::new(base);

        let json = serde_json::to_string(&tracking).unwrap();
        assert!(json.contains("\"baseVersion\""));
        assert!(json.contains("\"enabled\":true"));
    }

    // CRDT Tests

    #[test]
    fn test_crdt_format() {
        assert_eq!(CrdtFormat::Yjs.to_string(), "yjs");
        assert_eq!(CrdtFormat::Automerge.to_string(), "automerge");
        assert_eq!(CrdtFormat::DiamondTypes.to_string(), "diamond-types");

        let json = serde_json::to_string(&CrdtFormat::Yjs).unwrap();
        assert_eq!(json, "\"yjs\"");

        let deserialized: CrdtFormat = serde_json::from_str("\"automerge\"").unwrap();
        assert_eq!(deserialized, CrdtFormat::Automerge);
    }

    #[test]
    fn test_crdt_metadata() {
        let mut meta = CrdtMetadata::new("site1", 1);
        assert_eq!(meta.origin, "site1");
        assert_eq!(meta.seq, 1);
        assert_eq!(meta.clock.get("site1"), Some(&1));

        meta.increment("site1");
        assert_eq!(meta.seq, 2);
        assert_eq!(meta.clock.get("site1"), Some(&2));

        meta.increment("site2");
        assert_eq!(meta.clock.get("site2"), Some(&1));
    }

    #[test]
    fn test_crdt_metadata_merge() {
        let mut meta1 = CrdtMetadata::new("site1", 5);
        let mut meta2 = CrdtMetadata::new("site2", 3);
        meta2.clock.insert("site1".to_string(), 2);

        meta1.merge(&meta2);

        assert_eq!(meta1.clock.get("site1"), Some(&5)); // max(5, 2)
        assert_eq!(meta1.clock.get("site2"), Some(&3)); // from meta2
    }

    #[test]
    fn test_crdt_metadata_happened_before() {
        let meta1 = CrdtMetadata::new("site1", 1);
        let mut meta2 = CrdtMetadata::new("site1", 2);
        meta2.clock.insert("site1".to_string(), 2);

        assert!(meta1.happened_before(&meta2));
        assert!(!meta2.happened_before(&meta1));
    }

    #[test]
    fn test_text_crdt_metadata() {
        let meta = TextCrdtMetadata::from_text("Hello", "s1");

        assert_eq!(meta.len(), 5);
        assert!(!meta.is_empty());
        assert_eq!(meta.text(), "Hello");
        assert_eq!(meta.positions[0].id, "s1:1");
        assert_eq!(meta.positions[0].char, 'H');
    }

    #[test]
    fn test_sync_state() {
        let mut sync = SyncState::yjs()
            .with_version("13.6")
            .with_sync_version(1234);

        assert_eq!(sync.crdt_format, CrdtFormat::Yjs);
        assert_eq!(sync.crdt_version, Some("13.6".to_string()));
        assert_eq!(sync.sync_version, Some(1234));

        let peer = Peer::new("peer1").with_name("Alice");
        sync.add_peer(peer);

        assert_eq!(sync.peers.len(), 1);
        assert_eq!(sync.peers[0].name, Some("Alice".to_string()));

        sync.mark_synced();
        assert!(sync.last_sync.is_some());
    }

    #[test]
    fn test_sync_state_presets() {
        let yjs = SyncState::yjs();
        assert_eq!(yjs.crdt_format, CrdtFormat::Yjs);

        let automerge = SyncState::automerge();
        assert_eq!(automerge.crdt_format, CrdtFormat::Automerge);

        let diamond = SyncState::diamond_types();
        assert_eq!(diamond.crdt_format, CrdtFormat::DiamondTypes);
    }

    #[test]
    fn test_peer() {
        let mut peer = Peer::new("peer1").with_name("Alice");

        assert_eq!(peer.id, "peer1");
        assert_eq!(peer.name, Some("Alice".to_string()));

        let before = peer.last_seen;
        std::thread::sleep(std::time::Duration::from_millis(10));
        peer.touch();
        assert!(peer.last_seen > before);
    }

    #[test]
    fn test_revision_history() {
        let mut history = RevisionHistory::new();
        assert!(history.is_empty());
        assert_eq!(history.next_version(), 1);

        let doc_id: crate::DocumentId =
            "sha256:abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234"
                .parse()
                .unwrap();
        let author = Collaborator::new("Alice");

        let rev1 = Revision::new(1, doc_id.clone(), author.clone()).with_note("Initial draft");
        history.add(rev1);

        assert_eq!(history.len(), 1);
        assert_eq!(history.next_version(), 2);
        assert!(history.latest().is_some());
        assert_eq!(history.latest().unwrap().version, 1);
        assert!(history.get_version(1).is_some());
        assert!(history.get_version(2).is_none());
    }

    #[test]
    fn test_revision() {
        let doc_id: crate::DocumentId =
            "sha256:abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234"
                .parse()
                .unwrap();
        let author = Collaborator::new("Alice");

        let rev = Revision::new(1, doc_id, author)
            .with_note("Initial draft")
            .with_tag("draft");

        assert_eq!(rev.version, 1);
        assert_eq!(rev.note, Some("Initial draft".to_string()));
        assert_eq!(rev.tags, vec!["draft".to_string()]);
    }

    #[test]
    fn test_materialization_event() {
        let event = MaterializationEvent::cross_tool_exchange(
            "codex-tool/2.0",
            CrdtFormat::Yjs,
            CrdtFormat::Automerge,
        );

        assert_eq!(event.agent, "codex-tool/2.0");
        assert_eq!(event.from_crdt_format, Some(CrdtFormat::Yjs));
        assert_eq!(event.to_crdt_format, Some(CrdtFormat::Automerge));
        assert_eq!(event.reason, MaterializationReason::CrossToolExchange);
    }

    #[test]
    fn test_materialization_reason_display() {
        assert_eq!(
            MaterializationReason::CrossToolExchange.to_string(),
            "cross-tool-exchange"
        );
        assert_eq!(MaterializationReason::Export.to_string(), "export");
        assert_eq!(MaterializationReason::Archive.to_string(), "archive");
        assert_eq!(
            MaterializationReason::UserRequest.to_string(),
            "user-request"
        );
    }

    #[test]
    fn test_sync_state_serialization() {
        let sync = SyncState::yjs().with_version("13.6");
        let json = serde_json::to_string_pretty(&sync).unwrap();

        assert!(json.contains("\"crdtFormat\": \"yjs\""));
        assert!(json.contains("\"crdtVersion\": \"13.6\""));

        let deserialized: SyncState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.crdt_format, CrdtFormat::Yjs);
    }
}
