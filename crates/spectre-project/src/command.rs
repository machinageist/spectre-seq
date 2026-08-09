// Author: Jeff
// Date: 2026-07-12
// Description: Atomic project commands, grouped transactions, and bounded undo/redo history
// Notes: App-thread project mutation seam; never callback-reachable

use crate::ProjectDoc;
use std::collections::VecDeque;

// Command and history failures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandError {
    EmptyTransaction,
    InvalidProjectName,
    ZeroHistoryCapacity,
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::EmptyTransaction => "transaction must contain at least one command",
            Self::InvalidProjectName => "project name must contain a non-whitespace character",
            Self::ZeroHistoryCapacity => "history capacity must be greater than zero",
        };
        f.write_str(message)
    }
}

impl std::error::Error for CommandError {}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CommandKind {
    SetProjectName { name: String, validate: bool },
}

// One reversible project-model mutation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectCommand {
    kind: CommandKind,
}

impl ProjectCommand {
    // Create a validated project rename
    pub fn rename(name: impl Into<String>) -> Self {
        Self {
            kind: CommandKind::SetProjectName {
                name: name.into(),
                validate: true,
            },
        }
    }

    fn apply(&self, project: &mut ProjectDoc) -> Result<Self, CommandError> {
        match &self.kind {
            CommandKind::SetProjectName { name, validate } => {
                if *validate && name.trim().is_empty() {
                    return Err(CommandError::InvalidProjectName);
                }
                let previous = std::mem::replace(&mut project.name, name.clone());
                Ok(Self {
                    kind: CommandKind::SetProjectName {
                        name: previous,
                        validate: false,
                    },
                })
            }
        }
    }
}

// Ordered command group that applies and reverses atomically
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    commands: Vec<ProjectCommand>,
}

impl Transaction {
    // Reject empty edits that would pollute history
    pub fn new(commands: Vec<ProjectCommand>) -> Result<Self, CommandError> {
        if commands.is_empty() {
            return Err(CommandError::EmptyTransaction);
        }
        Ok(Self { commands })
    }

    // Build a one-command transaction
    pub fn single(command: ProjectCommand) -> Self {
        Self {
            commands: vec![command],
        }
    }

    fn execute(&self, project: &mut ProjectDoc) -> Result<Self, CommandError> {
        let mut inverses = Vec::with_capacity(self.commands.len());
        for command in &self.commands {
            match command.apply(project) {
                Ok(inverse) => inverses.push(inverse),
                Err(error) => {
                    for inverse in inverses.iter().rev() {
                        inverse
                            .apply(project)
                            .expect("generated inverse commands are infallible");
                    }
                    return Err(error);
                }
            }
        }
        inverses.reverse();
        Ok(Self { commands: inverses })
    }
}

// Bounded undo/redo stacks with oldest-edit eviction
#[derive(Debug)]
pub struct EditHistory {
    capacity: usize,
    undo: VecDeque<Transaction>,
    redo: VecDeque<Transaction>,
}

impl EditHistory {
    // Create history with an explicit nonzero bound
    pub fn new(capacity: usize) -> Result<Self, CommandError> {
        if capacity == 0 {
            return Err(CommandError::ZeroHistoryCapacity);
        }
        Ok(Self {
            capacity,
            undo: VecDeque::with_capacity(capacity),
            redo: VecDeque::with_capacity(capacity),
        })
    }

    // Apply one atomic edit and clear the abandoned redo branch
    pub fn apply(
        &mut self,
        project: &mut ProjectDoc,
        transaction: Transaction,
    ) -> Result<(), CommandError> {
        let inverse = transaction.execute(project)?;
        Self::push_bounded(&mut self.undo, inverse, self.capacity);
        self.redo.clear();
        Ok(())
    }

    // Reverse the latest edit; false means no edit was available
    pub fn undo(&mut self, project: &mut ProjectDoc) -> Result<bool, CommandError> {
        Self::transfer(&mut self.undo, &mut self.redo, project, self.capacity)
    }

    // Reapply the latest reversed edit; false means no edit was available
    pub fn redo(&mut self, project: &mut ProjectDoc) -> Result<bool, CommandError> {
        Self::transfer(&mut self.redo, &mut self.undo, project, self.capacity)
    }

    fn transfer(
        source: &mut VecDeque<Transaction>,
        destination: &mut VecDeque<Transaction>,
        project: &mut ProjectDoc,
        capacity: usize,
    ) -> Result<bool, CommandError> {
        let Some(transaction) = source.pop_back() else {
            return Ok(false);
        };
        match transaction.execute(project) {
            Ok(inverse) => {
                Self::push_bounded(destination, inverse, capacity);
                Ok(true)
            }
            Err(error) => {
                source.push_back(transaction);
                Err(error)
            }
        }
    }

    fn push_bounded(stack: &mut VecDeque<Transaction>, transaction: Transaction, capacity: usize) {
        if stack.len() == capacity {
            stack.pop_front();
        }
        stack.push_back(transaction);
    }
}
