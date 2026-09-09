use super::{
    CandidateRole as Role, ClassificationFailure as Failure, ClassificationLimits,
    ClassificationOutcome, ClassificationRecord, SymbolClassificationReport,
};
use crate::elf::dynamic::symbol_table::{
    Section,
    bounded::{SymbolObservationReport, SymbolOutcome},
};
use std::sync::Arc;
/// Role derivation never examines names, source bytes, relocations or other artifacts.
pub fn classify(
    input: Arc<SymbolObservationReport>,
    limits: ClassificationLimits,
) -> SymbolClassificationReport {
    let outcome = match input.outcome() {
        SymbolOutcome::Unavailable => ClassificationOutcome::Unavailable,
        SymbolOutcome::Failed(_) => ClassificationOutcome::Failed(Failure::PrerequisiteFailed),
        SymbolOutcome::Complete(symbols) => {
            let count = symbols.len() as u64;
            if count > limits.max_classifications {
                ClassificationOutcome::Failed(Failure::Budget {
                    count,
                    maximum: limits.max_classifications,
                })
            } else {
                let mut records = Vec::new();
                if records.try_reserve_exact(symbols.len()).is_err() {
                    ClassificationOutcome::Failed(Failure::Allocation { count })
                } else {
                    for symbol in symbols {
                        let fields = symbol.fields();
                        let role = if fields.index == 0 {
                            Role::NullSymbol
                        } else {
                            match fields.section {
                                Section::Undefined => Role::UndefinedCandidate,
                                Section::Index(_) => Role::DefinitionCandidate,
                                Section::Absolute
                                | Section::Common
                                | Section::Extended
                                | Section::Reserved(_) => Role::SpecialCandidate,
                            }
                        };
                        records.push(ClassificationRecord {
                            symbol_index: fields.index,
                            role,
                        });
                    }
                    ClassificationOutcome::Complete(records)
                }
            }
        }
    };
    SymbolClassificationReport {
        input,
        limits,
        outcome,
    }
}
