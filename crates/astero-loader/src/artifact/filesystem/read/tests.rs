use super::*;
use std::{error::Error, io::Cursor};
fn limits() -> AcquisitionLimits {
    AcquisitionLimits {
        max_bytes: 1_000_000,
        max_read_calls: 100,
    }
}
fn path() -> &'static Path {
    Path::new("synthetic-read-fixture")
}

#[test]
fn extent_sweep_distinguishes_exact_short_and_growing_reads() {
    for expected in 0..16 {
        for actual in 0..16 {
            let mut bytes = Cursor::new(vec![0xff; actual]);
            let result = read_exact_extent(&mut bytes, expected, limits(), path());
            if actual as u64 == expected {
                assert_eq!(result.unwrap(), vec![0xff; actual]);
            } else if (actual as u64) < expected {
                assert!(
                    matches!(result.unwrap_err().failure, Failure::ShortRead { expected: e, actual: a } if e==expected && a==actual as u64)
                );
            } else {
                assert!(
                    matches!(result.unwrap_err().failure, Failure::GrewDuringRead { expected: e, observed_at_least: a } if e==expected && a==expected+1)
                );
            }
        }
    }
}
struct Fragmented {
    bytes: Cursor<Vec<u8>>,
    interrupts: usize,
    calls: usize,
    largest: usize,
}
impl Read for Fragmented {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        self.calls += 1;
        self.largest = self.largest.max(out.len());
        if self.interrupts > 0 {
            self.interrupts -= 1;
            return Err(io::ErrorKind::Interrupted.into());
        }
        let length = out.len().min(1);
        self.bytes.read(&mut out[..length])
    }
}
#[test]
fn short_reads_and_interruptions_are_bounded_including_eof_probe() {
    for budget in 0..8 {
        let mut r = Fragmented {
            bytes: Cursor::new(vec![1, 2, 3]),
            interrupts: 2,
            calls: 0,
            largest: 0,
        };
        let result = read_exact_extent(
            &mut r,
            3,
            AcquisitionLimits {
                max_read_calls: budget,
                ..limits()
            },
            path(),
        );
        assert!(r.calls as u64 <= budget);
        if budget < 6 {
            assert!(matches!(
                result.unwrap_err().failure,
                Failure::ReadBudget { .. }
            ));
        } else {
            assert_eq!(result.unwrap(), [1, 2, 3]);
        }
    }
}
#[test]
fn byte_limits_and_unrepresentable_lengths_fail_before_reading() {
    let mut r = Cursor::new(vec![]);
    let e = read_exact_extent(
        &mut r,
        5,
        AcquisitionLimits {
            max_bytes: 4,
            ..limits()
        },
        path(),
    )
    .unwrap_err();
    assert!(matches!(
        e.failure,
        Failure::SizeLimit {
            observed: 5,
            maximum: 4
        }
    ));
    let e = read_exact_extent(
        &mut r,
        u64::MAX,
        AcquisitionLimits {
            max_bytes: u64::MAX,
            ..limits()
        },
        path(),
    )
    .unwrap_err();
    assert!(matches!(e.failure, Failure::LengthUnrepresentable { .. }));
    assert_eq!(r.position(), 0);
}
#[test]
fn io_failure_keeps_path_operation_and_underlying_error() {
    struct Denied;
    impl Read for Denied {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::ErrorKind::PermissionDenied.into())
        }
    }
    for expected in [0, 1] {
        let e = read_exact_extent(&mut Denied, expected, limits(), path()).unwrap_err();
        assert_eq!(e.path, path());
        assert_eq!(
            e.operation,
            if expected == 0 {
                Operation::ProbeEnd
            } else {
                Operation::Read
            }
        );
        assert_eq!(
            e.source()
                .unwrap()
                .downcast_ref::<io::Error>()
                .unwrap()
                .kind(),
            io::ErrorKind::PermissionDenied
        );
    }
}
#[test]
fn read_requests_are_chunk_bounded_and_final_size_changes_reject() {
    struct Measured {
        bytes: Cursor<Vec<u8>>,
        largest: usize,
        calls: usize,
    }
    impl Read for Measured {
        fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
            self.largest = self.largest.max(b.len());
            self.calls += 1;
            self.bytes.read(b)
        }
    }
    let n = CHUNK * 2 + 1;
    let mut r = Measured {
        bytes: Cursor::new(vec![7; n]),
        largest: 0,
        calls: 0,
    };
    assert_eq!(
        read_exact_extent(&mut r, n as u64, limits(), path())
            .unwrap()
            .len(),
        n
    );
    assert_eq!((r.largest, r.calls), (CHUNK, 4));
    for after in [0, 2] {
        let e = super::super::acquire::check_final_size(path(), 1, after).unwrap_err();
        assert!(matches!(e.failure,Failure::SizeChanged { before:1, after:a } if a==after));
    }
    assert!(super::super::acquire::check_final_size(path(), 1, 1).is_ok());
}
