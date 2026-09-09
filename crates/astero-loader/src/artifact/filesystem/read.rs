use super::{AcquisitionError, AcquisitionLimits, Failure, Operation};
use std::{
    io::{self, Read},
    path::Path,
};

const CHUNK: usize = 64 * 1024;

/// Private test seam: public callers always use a checked regular File, never arbitrary streams.
pub(super) fn read_exact_extent(
    reader: &mut impl Read,
    expected: u64,
    limits: AcquisitionLimits,
    path: &Path,
) -> Result<Vec<u8>, AcquisitionError> {
    let err = |op, failure| AcquisitionError::new(path, op, failure);
    if expected > limits.max_bytes {
        return Err(err(
            Operation::InspectHandle,
            Failure::SizeLimit {
                observed: expected,
                maximum: limits.max_bytes,
            },
        ));
    }
    let length = usize::try_from(expected)
        .ok()
        .filter(|n| *n <= isize::MAX as usize)
        .ok_or_else(|| {
            err(
                Operation::Allocate,
                Failure::LengthUnrepresentable { observed: expected },
            )
        })?;
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(length).map_err(|source| {
        err(
            Operation::Allocate,
            Failure::Allocation {
                requested: length,
                source,
            },
        )
    })?;
    bytes.resize(length, 0);
    let mut done = 0;
    let mut calls = 0;
    while done < length {
        let end = done + (length - done).min(CHUNK);
        match bounded_read(
            reader,
            &mut bytes[done..end],
            &mut calls,
            limits,
            path,
            Operation::Read,
            done as u64,
        )? {
            0 => {
                return Err(err(
                    Operation::Read,
                    Failure::ShortRead {
                        expected,
                        actual: done as u64,
                    },
                ));
            }
            n => done += n,
        }
    }
    // At most one excess byte, on the stack; never appended or accepted as a partial artifact.
    let mut probe = [0u8; 1];
    if bounded_read(
        reader,
        &mut probe,
        &mut calls,
        limits,
        path,
        Operation::ProbeEnd,
        expected,
    )? != 0
    {
        return Err(err(
            Operation::ProbeEnd,
            Failure::GrewDuringRead {
                expected,
                observed_at_least: expected + 1,
            },
        ));
    }
    Ok(bytes)
}
fn bounded_read(
    reader: &mut impl Read,
    buffer: &mut [u8],
    calls: &mut u64,
    limits: AcquisitionLimits,
    path: &Path,
    operation: Operation,
    completed: u64,
) -> Result<usize, AcquisitionError> {
    loop {
        if *calls == limits.max_read_calls {
            return Err(AcquisitionError::new(
                path,
                operation,
                Failure::ReadBudget {
                    maximum: limits.max_read_calls,
                    completed_bytes: completed,
                },
            ));
        }
        *calls += 1;
        match reader.read(buffer) {
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(AcquisitionError::new(path, operation, Failure::Io(e))),
            Ok(n) => return Ok(n),
        }
    }
}

#[cfg(test)]
mod tests;
