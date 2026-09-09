use super::{Classification as C, Reason as R};
use crate::elf::dynamic::{
    candidates::evidence::{CandidateEvidence, NameState},
    symbol_table::{Binding as B, Section as S, SymbolType as T, Visibility as V},
};
pub(in crate::elf::dynamic::candidates) fn classify(e: &CandidateEvidence<'_>) -> C {
    let s = e.symbol();
    if s.index == 0 {
        return C::Null;
    }
    let unknown = |reason| {
        if s.section == S::Undefined {
            C::UndefinedUnclassified(reason)
        } else {
            C::Unclassified(reason)
        }
    };
    match s.section {
        S::Absolute => return C::Absolute,
        S::Common => return C::Common,
        S::Extended | S::Reserved(_) => return unknown(R::SpecialSection),
        _ => {}
    }
    if matches!(s.binding, B::Unknown(_))
        || matches!(s.visibility, V::Unknown(_))
        || s.other & !7 != 0
    {
        return unknown(R::UnknownAttributes);
    }
    if matches!(
        s.symbol_type,
        T::Unknown(_) | T::Section | T::File | T::Common
    ) {
        return unknown(R::UnsupportedType);
    }
    if s.binding == B::Local && s.visibility == V::Protected {
        return unknown(R::ConflictingAttributes);
    }
    if s.section == S::Undefined {
        if s.binding == B::Local {
            return unknown(R::LocalBinding);
        }
        if s.visibility != V::Default {
            return unknown(R::NonExternalVisibility);
        }
        return C::ImportCandidate;
    }
    if s.binding == B::Local {
        return C::InternalDefined(R::LocalBinding);
    }
    if matches!(s.visibility, V::Hidden | V::Internal) {
        return C::InternalDefined(R::NonExternalVisibility);
    }
    match e.name_state() {
        NameState::Absent => unknown(R::MissingName),
        NameState::Empty => unknown(R::EmptyName),
        NameState::Utf8 | NameState::Bytes => C::ExportCandidate,
    }
}
