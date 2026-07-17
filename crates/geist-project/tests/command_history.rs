// Author: Jeff
// Date: 2026-07-12
// Description: Behavioral tests for project commands, atomic transactions, and bounded history
// Notes: R1 command seed stays off the realtime path

use geist_core::{IdGen, TempoMap, Transport};
use geist_project::{
    command::{CommandError, EditHistory, ProjectCommand, Transaction},
    ProjectDoc,
};
use serde_json::{json, Map};

fn project() -> ProjectDoc {
    let mut ids = IdGen::new(41);
    let mut unknown = Map::new();
    unknown.insert("future".into(), json!({"kept": true}));
    ProjectDoc {
        id: ids.next_id(),
        name: "Original".into(),
        tempo_map: TempoMap::constant(120.0).unwrap(),
        transport: Transport::new(),
        unknown,
    }
}

#[test]
fn rename_round_trips_without_touching_identity_or_unknown_fields() {
    let mut project = project();
    let id = project.id;
    let unknown = project.unknown.clone();
    let mut history = EditHistory::new(8).unwrap();

    history
        .apply(
            &mut project,
            Transaction::single(ProjectCommand::rename("Renamed")),
        )
        .unwrap();
    assert_eq!(project.name, "Renamed");

    assert!(history.undo(&mut project).unwrap());
    assert_eq!(project.name, "Original");
    assert_eq!(project.id, id);
    assert_eq!(project.unknown, unknown);

    assert!(history.redo(&mut project).unwrap());
    assert_eq!(project.name, "Renamed");
}

#[test]
fn failed_transaction_is_atomic() {
    let mut project = project();
    let original = project.clone();
    let mut history = EditHistory::new(8).unwrap();
    let transaction = Transaction::new(vec![
        ProjectCommand::rename("Temporary"),
        ProjectCommand::rename("   "),
    ])
    .unwrap();

    assert!(matches!(
        history.apply(&mut project, transaction),
        Err(CommandError::InvalidProjectName)
    ));
    assert_eq!(project, original);
    assert!(!history.undo(&mut project).unwrap());
}

#[test]
fn new_edit_clears_redo_history() {
    let mut project = project();
    let mut history = EditHistory::new(8).unwrap();
    history
        .apply(
            &mut project,
            Transaction::single(ProjectCommand::rename("First")),
        )
        .unwrap();
    assert!(history.undo(&mut project).unwrap());

    history
        .apply(
            &mut project,
            Transaction::single(ProjectCommand::rename("Branch")),
        )
        .unwrap();
    assert!(!history.redo(&mut project).unwrap());
    assert_eq!(project.name, "Branch");
}

#[test]
fn bounded_history_discards_the_oldest_edit() {
    let mut project = project();
    let mut history = EditHistory::new(2).unwrap();
    for name in ["One", "Two", "Three"] {
        history
            .apply(
                &mut project,
                Transaction::single(ProjectCommand::rename(name)),
            )
            .unwrap();
    }

    assert!(history.undo(&mut project).unwrap());
    assert_eq!(project.name, "Two");
    assert!(history.undo(&mut project).unwrap());
    assert_eq!(project.name, "One");
    assert!(!history.undo(&mut project).unwrap());
}

#[test]
fn empty_transactions_and_zero_capacity_are_rejected() {
    assert!(matches!(
        Transaction::new(Vec::new()),
        Err(CommandError::EmptyTransaction)
    ));
    assert!(matches!(
        EditHistory::new(0),
        Err(CommandError::ZeroHistoryCapacity)
    ));
}
