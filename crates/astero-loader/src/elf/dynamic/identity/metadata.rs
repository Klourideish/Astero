use super::*;
pub(super) fn kind(tag: i64) -> DescriptorKind {
    match tag {
        0x61000043 => DescriptorKind::Module,
        0x61000045 => DescriptorKind::NeededModule,
        0x61000047 => DescriptorKind::ExportLibrary,
        0x61000049 => DescriptorKind::ImportLibrary,
        _ => DescriptorKind::Unsupported,
    }
}
// Preserve all declarations. Identical duplicates corroborate only the same local hypothesis;
// any differing name/version/kind leaves a conflict. Never prefer a provider.
fn context(id: u16, libraries: bool, records: &[IdentityDescriptor]) -> ContextEvidence {
    let mut first: Option<&IdentityDescriptor> = None;
    let mut count = 0;
    let mut conflict = false;
    for d in records.iter().filter(|d| {
        d.id == Some(id)
            && matches!(
                d.kind,
                DescriptorKind::ImportLibrary | DescriptorKind::ExportLibrary
            ) == libraries
    }) {
        count += 1;
        if let Some(f) = first {
            if f.raw != d.raw || f.kind != d.kind {
                conflict = true;
            }
        } else {
            first = Some(d);
        }
    }
    match first {
        None => ContextEvidence::Missing { id },
        Some(_) if conflict => ContextEvidence::Conflict {
            id,
            declarations: count,
        },
        Some(d) => ContextEvidence::Hypothesis {
            id,
            dynamic_index: d.dynamic_index,
            declarations: count,
        },
    }
}
pub(super) fn correlate(name: Option<&[u8]>, records: &[IdentityDescriptor]) -> NameEvidence {
    let Some(bytes) = name else {
        return NameEvidence::Unnamed;
    };
    if bytes.len() != 11 && !bytes.contains(&b'#') {
        return NameEvidence::PlainOrRaw;
    }
    let mut parts = bytes.split(|b| *b == b'#');
    let token = parts.next().expect("split has one part");
    let nid = match codec::decode_nid(token) {
        Ok(n) => n,
        Err(e) => return NameEvidence::Candidate(e),
    };
    let (library, module) = match (parts.next(), parts.next(), parts.next()) {
        (None, None, None) => return NameEvidence::Candidate(codec::EncodingError::Context),
        (Some(l), Some(m), None) => match (codec::decode_context(l), codec::decode_context(m)) {
            (Ok(l), Ok(m)) => (context(l, true, records), context(m, false, records)),
            _ => return NameEvidence::Candidate(codec::EncodingError::Context),
        },
        _ => return NameEvidence::Candidate(codec::EncodingError::Context),
    };
    NameEvidence::Encoded {
        nid,
        library,
        module,
    }
}
