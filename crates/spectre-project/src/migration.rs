// Author: Jeff
// Date: 2026-08-31
// Description: Explicit project-schema migration to the current readable document
// Notes: Migration is app-thread work. It preserves the decoded envelope whole and changes only
//   fields whose meaning is defined by the source and destination schemas.

use crate::{validate_envelope, ProjectEnvelope, ValidationError, SCHEMA_VERSION};

// Evidence returned with every migration, including a no-op on an already-current document.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationReceipt {
    pub envelope: ProjectEnvelope,
    pub from_schema: u32,
    pub to_schema: u32,
}

impl MigrationReceipt {
    pub fn changed(&self) -> bool {
        self.from_schema != self.to_schema
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationError {
    UnsupportedSchema { found: u32 },
    Invalid(ValidationError),
    SchemaOneCarriesSchemaTwoState,
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedSchema { found } => {
                write!(formatter, "project schema {found} has no migration in this build")
            }
            Self::Invalid(source) => write!(formatter, "project is invalid before migration: {source}"),
            Self::SchemaOneCarriesSchemaTwoState => formatter.write_str(
                "schema 1 project carries schema 2 collections or view state and cannot be migrated safely",
            ),
        }
    }
}

impl std::error::Error for MigrationError {}

// Upgrade one already-decoded envelope. Decode remains non-mutating so crate-level rewrites can
// preserve an old fixture byte-for-byte; the product calls this explicitly when adopting a file.
pub fn migrate_to_current(
    mut envelope: ProjectEnvelope,
) -> Result<MigrationReceipt, MigrationError> {
    validate_envelope(&envelope).map_err(MigrationError::Invalid)?;
    let from_schema = envelope.schema_version;

    match from_schema {
        SCHEMA_VERSION => {}
        2 => {
            // Schema 3 replaces the single insert slot with an ordered chain. The slot decodes
            // into a legacy field that nothing reads, so a schema-2 file whose tracks are not
            // migrated would silently lose every effect -- which is what the alpha fixture's own
            // test caught the first time this ran
            adopt_legacy_inserts(&mut envelope);
            envelope.schema_version = SCHEMA_VERSION;
        }
        1 => {
            // Schema 1 predates every collection and persisted view. Refuse a mislabeled file
            // rather than seed an ID generator that could collide with hidden schema-2 objects.
            if !envelope.project.tracks.is_empty()
                || !envelope.project.devices.is_empty()
                || envelope.project.view != crate::ViewDoc::default()
                || envelope.project.id_gen_state != 0
            {
                return Err(MigrationError::SchemaOneCarriesSchemaTwoState);
            }
            // This is the same deterministic fallback the R4 shell used implicitly. A legitimate
            // schema-1 project has no other persisted object IDs, so the next generated identity
            // cannot duplicate an existing collection member.
            envelope.project.id_gen_state = envelope.project.id.raw();
            // A schema-1 document carries no tracks at all, so this is a no-op by construction.
            // Called anyway so a future schema-1 variant that did carry one cannot slip past
            adopt_legacy_inserts(&mut envelope);
            envelope.schema_version = SCHEMA_VERSION;
        }
        found => return Err(MigrationError::UnsupportedSchema { found }),
    }

    validate_envelope(&envelope).map_err(MigrationError::Invalid)?;
    Ok(MigrationReceipt {
        envelope,
        from_schema,
        to_schema: SCHEMA_VERSION,
    })
}

// Move every track's schema-2 insert slot into its schema-3 chain. Idempotent: a track whose
// chain is already populated keeps it, and a track with no legacy slot is untouched
fn adopt_legacy_inserts(envelope: &mut ProjectEnvelope) {
    for track in envelope.project.tracks.tracks_mut() {
        track.adopt_legacy_insert();
    }
}
