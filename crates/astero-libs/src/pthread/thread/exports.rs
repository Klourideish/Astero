//! Exact libkernel lifecycle identities and checked guest copies. No native execution in libs.
use crate::pthread::attributes::lifecycle::{read64, write};
use astero_hle::{calls::memory::GuestMemory, dispatch::prepared::*};
use astero_kernel::threading::thread::{
    attributes::{AttributeTable, Attributes},
    lifecycle::{Error, Result, Thread, ThreadTable},
};
use std::sync::{Arc, Mutex};
pub trait Spawner: Send + Sync {
    fn create(
        &self,
        attr: Attributes,
        start: u64,
        argument: u64,
        name: Vec<u8>,
        output: u64,
        memory: &mut dyn GuestMemory,
    ) -> Result;
}
#[derive(Clone, Copy)]
pub struct Export {
    pub name: &'static str,
    pub nid: u64,
    pub operation: &'static str,
    pub sce: bool,
}
// Exact numeric identities routed through PS5Rust threading/nids.rs; no name hashing fallback.
pub const EXPORTS: &[Export] = &[
    Export {
        name: "SCE_PTHREAD_CREATE",
        nid: 0xE9482DC15FB4CDBE,
        operation: "CREATE",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_JOIN",
        nid: 0xA27358F41CA7FD6F,
        operation: "JOIN",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_DETACH",
        nid: 0xE2A1AB47A7A83FD6,
        operation: "DETACH",
        sce: true,
    },
    Export {
        name: "PTHREAD_CREATE",
        nid: 0x3B184807C2C1FCF4,
        operation: "CREATE",
        sce: false,
    },
    Export {
        name: "PTHREAD_CREATE_NAME_NP",
        nid: 0x2668BEF70F6ED04E,
        operation: "CREATE_NAME_NP",
        sce: false,
    },
    Export {
        name: "PTHREAD_JOIN",
        nid: 0x87D09C3F7274A153,
        operation: "JOIN",
        sce: false,
    },
    Export {
        name: "PTHREAD_DETACH",
        nid: 0xF94D51E16B57BE87,
        operation: "DETACH",
        sce: false,
    },
    Export {
        name: "PTHREAD_EXIT",
        nid: 0x149AD3E4BB940405,
        operation: "EXIT",
        sce: false,
    },
    Export {
        name: "SCE_PTHREAD_EXIT",
        nid: 0xDE483BAD3D0D408B,
        operation: "EXIT",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_GETTHREADID",
        nid: 0x108FF9FE396AD9D1,
        operation: "GETTHREADID",
        sce: true,
    },
    Export {
        name: "PTHREAD_GETTHREADID_NP",
        nid: 0xDDEAACDFB1BBE3FB,
        operation: "GETTHREADID_NP",
        sce: false,
    },
    Export {
        name: "SCE_PTHREAD_ATTR_INIT",
        nid: 0x9EC628351CB0C0D8,
        operation: "ATTR_INIT",
        sce: true,
    },
    Export {
        name: "PTHREAD_ATTR_INIT",
        nid: 0xC2D92DFED791D6CA,
        operation: "ATTR_INIT",
        sce: false,
    },
    Export {
        name: "SCE_PTHREAD_ATTR_SETSTACKSIZE",
        nid: 0x5135F325B5A18531,
        operation: "ATTR_SETSTACKSIZE",
        sce: true,
    },
    Export {
        name: "PTHREAD_ATTR_SETSTACKSIZE",
        nid: 0xD90D33EAB9C1AD31,
        operation: "ATTR_SETSTACKSIZE",
        sce: false,
    },
    Export {
        name: "SCE_PTHREAD_ATTR_SETSCHEDPARAM",
        nid: 0x0F3112F61405E1FE,
        operation: "ATTR_SETSCHEDPARAM",
        sce: true,
    },
    Export {
        name: "PTHREAD_ATTR_SETSCHEDPARAM",
        nid: 0x7AE291826D159F63,
        operation: "ATTR_SETSCHEDPARAM",
        sce: false,
    },
    Export {
        name: "SCE_PTHREAD_ATTR_SETSCHEDPOLICY",
        nid: 0xE3E87D133C0A1782,
        operation: "ATTR_SETSCHEDPOLICY",
        sce: true,
    },
    Export {
        name: "PTHREAD_ATTR_SETSCHEDPOLICY",
        nid: 0x25AACC232F242846,
        operation: "ATTR_SETSCHEDPOLICY",
        sce: false,
    },
    Export {
        name: "SCE_PTHREAD_ATTR_GETSCHEDPARAM",
        nid: 0x1573D61CD93C39FD,
        operation: "ATTR_GETSCHEDPARAM",
        sce: true,
    },
    Export {
        name: "PTHREAD_ATTR_GETSCHEDPARAM",
        nid: 0xAA593DA522EC5263,
        operation: "ATTR_GETSCHEDPARAM",
        sce: false,
    },
    Export {
        name: "SCE_PTHREAD_ATTR_SETINHERITSCHED",
        nid: 0x7976D44A911A4EC0,
        operation: "ATTR_SETINHERITSCHED",
        sce: true,
    },
    Export {
        name: "PTHREAD_ATTR_SETINHERITSCHED",
        nid: 0xED99406A411FD108,
        operation: "ATTR_SETINHERITSCHED",
        sce: false,
    },
    Export {
        name: "SCE_PTHREAD_ATTR_SETDETACHSTATE",
        nid: 0xFD6ADEA6BB6ED10B,
        operation: "ATTR_SETDETACHSTATE",
        sce: true,
    },
    Export {
        name: "PTHREAD_ATTR_SETDETACHSTATE",
        nid: 0x13EB72A37969E4BC,
        operation: "ATTR_SETDETACHSTATE",
        sce: false,
    },
    Export {
        name: "SCE_PTHREAD_ATTR_GETSTACKSIZE",
        nid: 0xFDF03EED99460D0B,
        operation: "ATTR_GETSTACKSIZE",
        sce: true,
    },
    Export {
        name: "PTHREAD_ATTR_GETSTACKSIZE",
        nid: 0xD2A3AD091FD91DC9,
        operation: "ATTR_GETSTACKSIZE",
        sce: false,
    },
    Export {
        name: "SCE_PTHREAD_ATTR_SETSTACK",
        nid: 0x06F9_FBE2_F8FA_A0BA,
        operation: "ATTR_SETSTACK",
        sce: true,
    },
    Export {
        name: "PTHREAD_ATTR_SETSTACK",
        nid: 0xFD2A_DB5E_9191_D5FD,
        operation: "ATTR_SETSTACK",
        sce: false,
    },
    Export {
        name: "SCE_PTHREAD_ATTR_GETSTACK",
        nid: 0xFEAB_8F6B_8484_254C,
        operation: "ATTR_GETSTACK",
        sce: true,
    },
    Export {
        name: "PTHREAD_ATTR_GETSTACK",
        nid: 0xBD09_B87C_312C_5A2F,
        operation: "ATTR_GETSTACK",
        sce: false,
    },
    Export {
        name: "SCE_PTHREAD_ATTR_SETSCOPE",
        nid: 0x61D6_5F11_97D1_9CF9,
        operation: "ATTR_SETSCOPE",
        sce: true,
    },
    Export {
        name: "PTHREAD_ATTR_SETSCOPE",
        nid: 0xC5EB_2695_223F_2822,
        operation: "ATTR_SETSCOPE",
        sce: false,
    },
    Export {
        name: "SCE_PTHREAD_ATTR_GETSCOPE",
        nid: 0xFBB0_7600_428A_9ECF,
        operation: "ATTR_GETSCOPE",
        sce: true,
    },
    Export {
        name: "PTHREAD_ATTR_GETSCOPE",
        nid: 0x7B61_BE71_D124_3A65,
        operation: "ATTR_GETSCOPE",
        sce: false,
    },
    Export {
        name: "PTHREAD_ATTR_GETDETACHSTATE",
        nid: 0x5544F5652AC74F42,
        operation: "ATTR_GETDETACHSTATE",
        sce: false,
    },
    Export {
        name: "PTHREAD_ATTR_GETGUARDSIZE",
        nid: 0x24D91556C54398E9,
        operation: "ATTR_GETGUARDSIZE",
        sce: false,
    },
    Export {
        name: "PTHREAD_ATTR_GETINHERITSCHED",
        nid: 0xA0B8CFA942A1CDEB,
        operation: "ATTR_GETINHERITSCHED",
        sce: false,
    },
    Export {
        name: "PTHREAD_ATTR_GETSCHEDPOLICY",
        nid: 0x46D2D157FA414D36,
        operation: "ATTR_GETSCHEDPOLICY",
        sce: false,
    },
    Export {
        name: "PTHREAD_ATTR_GETSTACKADDR",
        nid: 0x0F198831443FC176,
        operation: "ATTR_GETSTACKADDR",
        sce: false,
    },
    Export {
        name: "PTHREAD_ATTR_SETGUARDSIZE",
        nid: 0x24AC86DD25B2035D,
        operation: "ATTR_SETGUARDSIZE",
        sce: false,
    },
    Export {
        name: "PTHREAD_ATTR_SETSTACKADDR",
        nid: 0xB2E0AB11BAF4C484,
        operation: "ATTR_SETSTACKADDR",
        sce: false,
    },
    Export {
        name: "SCE_PTHREAD_ATTR_GETDETACHSTATE",
        nid: 0x25A44CCBE41CA5E5,
        operation: "ATTR_GETDETACHSTATE",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_ATTR_GETGUARDSIZE",
        nid: 0xB711ED9E027E7B27,
        operation: "ATTR_GETGUARDSIZE",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_ATTR_GETINHERITSCHED",
        nid: 0x96930FF0786405B8,
        operation: "ATTR_GETINHERITSCHED",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_ATTR_GETSCHEDPOLICY",
        nid: 0x34CC8843D5A059B5,
        operation: "ATTR_GETSCHEDPOLICY",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_ATTR_GETSTACKADDR",
        nid: 0x46EDFA7E24ED2730,
        operation: "ATTR_GETSTACKADDR",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_ATTR_SETGUARDSIZE",
        nid: 0x125F9C436D03CA75,
        operation: "ATTR_SETGUARDSIZE",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_ATTR_SETSTACKADDR",
        nid: 0x17EC9F99DB88041F,
        operation: "ATTR_SETSTACKADDR",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_ATTR_DESTROY",
        nid: 0xEB6282C04326CDC3,
        operation: "ATTR_DESTROY",
        sce: true,
    },
    Export {
        name: "PTHREAD_ATTR_DESTROY",
        nid: 0xCC77_2163_C7ED_E699,
        operation: "ATTR_DESTROY",
        sce: false,
    },
    Export {
        name: "SCE_PTHREAD_SELF",
        nid: 0x688F8E782CFCC6B4,
        operation: "SELF",
        sce: true,
    },
    Export {
        name: "PTHREAD_SELF",
        nid: 0x128B51F1ADC049FE,
        operation: "SELF",
        sce: false,
    },
    Export {
        name: "SCE_PTHREAD_EQUAL",
        nid: 0xDCFB55EA9DD0357E,
        operation: "EQUAL",
        sce: true,
    },
    Export {
        name: "PTHREAD_EQUAL",
        nid: 0xED7976E7B33854D2,
        operation: "EQUAL",
        sce: false,
    },
    Export {
        name: "SCE_PTHREAD_RENAME",
        nid: 0x181518EF2C1D50B1,
        operation: "RENAME",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_GETNAME",
        nid: 0x1E8C3B07C39EB7A9,
        operation: "GETNAME",
        sce: true,
    },
    Export {
        name: "PTHREAD_GETNAME_NP",
        nid: 0xF47CDF85DB444A2A,
        operation: "GETNAME_NP",
        sce: false,
    },
    Export {
        name: "SCE_PTHREAD_SET_NAME",
        nid: 0x5DE4EAC3ED19975D,
        operation: "SET_NAME",
        sce: true,
    },
    Export {
        name: "PTHREAD_RENAME_NP",
        nid: 0xF6FC8FE99EDBAB37,
        operation: "RENAME_NP",
        sce: false,
    },
    Export {
        name: "PTHREAD_SET_NAME_NP",
        nid: 0xA31329F2E3EA6BE5,
        operation: "SET_NAME_NP",
        sce: false,
    },
];
fn name(m: &dyn GuestMemory, a: u64) -> Result<Vec<u8>> {
    if a == 0 {
        return Ok(Vec::new());
    }
    let mut bytes = Vec::new();
    for n in 0..256u64 {
        let b = m
            .read(a.checked_add(n).ok_or(Error::Invalid)?, 1)
            .map_err(|_| Error::Invalid)?;
        let b = *b.first().ok_or(Error::Invalid)?;
        if b == 0 {
            bytes.truncate(31);
            return Ok(bytes);
        }
        bytes.push(b);
    }
    Err(Error::Invalid)
}
fn code(e: Error) -> u64 {
    match e {
        Error::NoSuchThread => 3,
        Error::Interrupted => 4,
        Error::Fault => 14,
        Error::Invalid => 22,
        Error::Deadlock => 11,
        Error::Busy => 16,
        Error::Capacity => 12,
        Error::Unsupported => 45,
    }
}
pub fn registrations(
    attrs: Arc<AttributeTable>,
    table: Arc<ThreadTable>,
    spawn: Arc<dyn Spawner>,
    caller: Thread,
    exit: Arc<Mutex<Option<u64>>>,
) -> Vec<Registration> {
    EXPORTS
        .iter()
        .map(|e| {
            let (attrs, table, spawn, exit) =
                (attrs.clone(), table.clone(), spawn.clone(), exit.clone());
            Registration {
                key: ProviderKey {
                    nid: e.nid,
                    library: b"libkernel".to_vec(),
                    module: b"libkernel".to_vec(),
                },
                kind: ProviderKind::HleImplementation,
                handler: Some(Box::new(move |f, m| {
                    let a = f.arguments;
                    if e.operation == "EXIT" {
                        *exit.lock().unwrap_or_else(|p| p.into_inner()) = Some(a[0]);
                        f.rax = a[0];
                        return CallResult::StopRequested;
                    }
                    let result: Result<u64> = (|| {
                        match e.operation {
                            "SELF" | "GETTHREADID" | "GETTHREADID_NP" => return Ok(caller.0),
                            "EQUAL" => return Ok(u64::from(a[0] == a[1])),
                            "CREATE" | "CREATE_NAME_NP" => {
                                let attr = if a[1] == 0 {
                                    Attributes::default()
                                } else {
                                    attrs.get(read64(m, a[1])?, a[1])?
                                };
                                let n = if e.sce || e.operation == "CREATE_NAME_NP" {
                                    name(m, a[4])?
                                } else {
                                    Vec::new()
                                };
                                spawn.create(attr, a[2], a[3], n, a[0], m)?;
                                return Ok(0);
                            }
                            "JOIN" => {
                                if a[1] != 0 {
                                    let old = m.read(a[1], 8).map_err(|_| Error::Invalid)?;
                                    write(m, a[1], &old)?;
                                }
                                let value = table.join(caller, Thread(a[0]))?;
                                if a[1] != 0 {
                                    write(m, a[1], &value.to_le_bytes())?;
                                }
                                return Ok(0);
                            }
                            "DETACH" => {
                                table.detach(Thread(a[0]))?;
                                return Ok(0);
                            }
                            "RENAME" | "RENAME_NP" | "SET_NAME" | "SET_NAME_NP" => {
                                if a[1] != 0 {
                                    table.rename(Thread(a[0]), name(m, a[1])?)?;
                                } else if !table.snapshot().iter().any(|r| r.id == Thread(a[0])) {
                                    return Err(Error::NoSuchThread);
                                }
                                return Ok(0);
                            }
                            "GETNAME" | "GETNAME_NP" => {
                                let r = table
                                    .snapshot()
                                    .into_iter()
                                    .find(|r| r.id == Thread(a[0]))
                                    .ok_or(Error::NoSuchThread)?;
                                let mut bytes = [0; 32];
                                bytes[..r.name.len()].copy_from_slice(&r.name);
                                write(m, a[1], &bytes).map_err(|_| Error::Fault)?;
                                return Ok(0);
                            }
                            _ => {}
                        }
                        crate::pthread::attributes::lifecycle::invoke(e.operation, a, &attrs, m)
                    })();
                    f.rax = match result {
                        Ok(v) => v,
                        Err(err) => {
                            let c = if err == Error::Capacity
                                && !e.sce
                                && matches!(e.operation, "CREATE" | "CREATE_NAME_NP")
                            {
                                35
                            } else {
                                code(err)
                            };
                            if err == Error::Interrupted {
                                return CallResult::StopRequested;
                            }
                            // Prototype/firmware getname is a raw-error two-argument helper even
                            // under its sce spelling; the ordinary SCE lifecycle wrappers encode.
                            if e.sce && e.operation != "GETNAME" {
                                c | 0x80020000
                            } else {
                                c
                            }
                        }
                    };
                    CallResult::Returned
                })),
            }
        })
        .collect()
}
