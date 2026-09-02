// Author: Jeff
// Date: 2026-08-31
// Description: R5 schema-migration evidence for identity and forward-field preservation
// Notes: Uses the checked-in schema-1 fixture rather than a schema invented inside the test.

use spectre_core::IdGen;
use spectre_project::{from_bytes, migrate_to_current, to_bytes, SCHEMA_VERSION};

#[test]
fn schema_one_migrates_explicitly_without_changing_identity_or_unknown_fields() {
    let source = from_bytes(include_bytes!("fixtures/r1-canonical.json")).expect("fixture decodes");
    assert_eq!(source.schema_version, 1);
    let project_id = source.project.id;
    let envelope_unknown = source.unknown.clone();
    let project_unknown = source.project.unknown.clone();

    let receipt = migrate_to_current(source).expect("schema 1 migrates");
    assert!(receipt.changed());
    assert_eq!(receipt.from_schema, 1);
    assert_eq!(receipt.to_schema, SCHEMA_VERSION);
    assert_eq!(receipt.envelope.project.id, project_id);
    assert_eq!(receipt.envelope.unknown, envelope_unknown);
    assert_eq!(receipt.envelope.project.unknown, project_unknown);
    assert_eq!(receipt.envelope.project.id_gen_state, project_id.raw());

    let decoded =
        from_bytes(&to_bytes(&receipt.envelope).unwrap()).expect("migration writes current");
    assert_eq!(decoded, receipt.envelope);

    // Schema 1 carries no object collection. Resuming from the migration seed therefore cannot
    // duplicate the one persisted identity that exists.
    let mut ids = IdGen::new(decoded.project.id_gen_state);
    assert_ne!(ids.next_id(), project_id);
}

#[test]
fn migrating_current_schema_is_an_exact_no_op() {
    let mut source = from_bytes(include_bytes!("fixtures/r1-canonical.json")).unwrap();
    source.schema_version = SCHEMA_VERSION;
    source.project.id_gen_state = source.project.id.raw();
    let expected = source.clone();

    let receipt = migrate_to_current(source).expect("current schema is accepted");
    assert!(!receipt.changed());
    assert_eq!(receipt.envelope, expected);
}

#[test]
fn a_mislabeled_schema_one_file_with_schema_two_state_is_refused() {
    let mut source = from_bytes(include_bytes!("fixtures/r1-canonical.json")).unwrap();
    source.project.id_gen_state = 7;

    let error = migrate_to_current(source).unwrap_err();
    assert!(matches!(
        error,
        spectre_project::MigrationError::SchemaOneCarriesSchemaTwoState
    ));
}
