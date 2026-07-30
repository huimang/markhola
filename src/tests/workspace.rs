use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::document::ActiveDocument;
use crate::file_io::{directory_base_url, save_markdown};

use super::{DocumentWorkspace, WorkspaceOpenResult};

fn document(id: u64, path: &str) -> ActiveDocument {
    ActiveDocument::open_with_id(
        id,
        PathBuf::from(path),
        format!("# {path}\ncontent"),
        "file:///tmp/".to_string(),
    )
}

fn temp_markdown_path(name: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("markhola-workspace-{name}-{stamp}.md"))
}

#[test]
fn opens_multiple_documents_and_tracks_active_one() {
    let mut workspace = DocumentWorkspace::new();

    assert_eq!(
        workspace.open_document(document(1, "/tmp/one.md")),
        WorkspaceOpenResult::OpenedNew(1)
    );
    assert_eq!(
        workspace.open_document(document(2, "/tmp/two.md")),
        WorkspaceOpenResult::OpenedNew(2)
    );

    assert_eq!(workspace.active_document_id(), Some(2));
    assert_eq!(workspace.tab_snapshots().len(), 2);
}

#[test]
fn blank_document_numbers_increment_independently_from_document_ids() {
    let mut workspace = DocumentWorkspace::new();

    assert_eq!(workspace.next_blank_document_number(), 1);
    assert_eq!(workspace.next_document_id(), 1);
    assert_eq!(workspace.next_document_id(), 2);
    assert_eq!(workspace.next_blank_document_number(), 2);
    assert_eq!(workspace.next_blank_document_number(), 3);
}

#[test]
fn reopening_same_path_activates_existing_document() {
    let mut workspace = DocumentWorkspace::new();

    workspace.open_document(document(1, "/tmp/one.md"));
    let result = workspace.open_document(document(2, "/tmp/one.md"));

    assert_eq!(result, WorkspaceOpenResult::ActivatedExisting(1));
    assert_eq!(workspace.tab_snapshots().len(), 1);
    assert_eq!(workspace.active_document_id(), Some(1));
}

#[test]
fn closing_active_document_selects_neighbor() {
    let mut workspace = DocumentWorkspace::new();

    workspace.open_document(document(1, "/tmp/one.md"));
    workspace.open_document(document(2, "/tmp/two.md"));
    workspace.open_document(document(3, "/tmp/three.md"));

    workspace.activate_document(2);
    let closed = workspace.close_document(2).unwrap();

    assert_eq!(closed.id(), 2);
    assert_eq!(workspace.active_document_id(), Some(3));
    assert_eq!(workspace.tab_snapshots().len(), 2);
}

#[test]
fn find_by_path_excluding_skips_the_requested_document() {
    let mut workspace = DocumentWorkspace::new();

    workspace.open_document(document(1, "/tmp/one.md"));
    workspace.open_document(document(2, "/tmp/two.md"));

    assert_eq!(
        workspace.find_by_path_excluding(Path::new("/tmp/one.md"), 1),
        None
    );
    assert_eq!(
        workspace.find_by_path_excluding(Path::new("/tmp/two.md"), 1),
        Some(2)
    );
}

#[test]
fn save_as_repath_updates_workspace_lookup_to_new_canonical_path() {
    let source = temp_markdown_path("source");
    let target = temp_markdown_path("target");
    save_markdown(&source, "# source").unwrap();

    let mut workspace = DocumentWorkspace::new();
    workspace.open_document(document(1, source.to_str().unwrap()));

    let base_url = directory_base_url(&target).unwrap();
    save_markdown(&target, "# source").unwrap();
    workspace
        .active_document_mut()
        .unwrap()
        .replace_file_path(target.clone(), base_url);

    assert_eq!(workspace.find_by_path(&target), Some(1));
    assert_eq!(workspace.find_by_path(&source), None);

    let _ = std::fs::remove_file(source);
    let _ = std::fs::remove_file(target);
}
