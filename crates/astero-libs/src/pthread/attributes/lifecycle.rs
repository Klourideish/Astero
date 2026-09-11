//! Guest general pthread attribute operations; kernel retains exact slot identity and values.
use astero_hle::calls::memory::GuestMemory;
use astero_kernel::threading::thread::{
    attributes::AttributeTable,
    lifecycle::{Error, Result},
};
pub(crate) fn read64(m: &dyn GuestMemory, address: u64) -> Result<u64> {
    Ok(u64::from_le_bytes(
        m.read(address, 8)
            .map_err(|_| Error::Invalid)?
            .try_into()
            .map_err(|_| Error::Invalid)?,
    ))
}
pub(crate) fn write(m: &mut dyn GuestMemory, address: u64, b: &[u8]) -> Result {
    if address == 0 {
        return Err(Error::Invalid);
    }
    m.write(address, b).map_err(|_| Error::Invalid)
}
pub(crate) fn invoke(
    operation: &str,
    a: [u64; 6],
    attrs: &AttributeTable,
    m: &mut dyn GuestMemory,
) -> Result<u64> {
    if operation == "ATTR_INIT" {
        let id = attrs.create(a[0])?;
        if let Err(e) = write(m, a[0], &id.to_le_bytes()) {
            let _ = attrs.destroy(id, a[0]);
            return Err(e);
        }
        return Ok(0);
    }
    let id = read64(m, a[0])?;
    let mut attr = attrs.get(id, a[0])?;
    match operation {
        "ATTR_DESTROY" => {
            write(m, a[0], &0u64.to_le_bytes())?;
            attrs.destroy(id, a[0])?;
        }
        "ATTR_SETSTACKSIZE" => {
            attr.stack_size = a[1];
            attrs.set(id, a[0], attr)?;
        }
        "ATTR_SETGUARDSIZE" => {
            attr.guard_size = a[1];
            attrs.set(id, a[0], attr)?;
        }
        "ATTR_SETSTACKADDR" | "ATTR_SETSTACK" => {
            return Err(Error::Unsupported);
        }
        "ATTR_SETDETACHSTATE" => {
            if a[1] > 1 {
                return Err(Error::Invalid);
            }
            attr.detached = a[1] == 1;
            attrs.set(id, a[0], attr)?;
        }
        "ATTR_SETINHERITSCHED" => {
            attr.inherit = a[1] as i32;
            attrs.set(id, a[0], attr)?;
        }
        "ATTR_SETSCHEDPOLICY" => {
            attr.policy = a[1] as i32;
            attrs.set(id, a[0], attr)?;
        }
        "ATTR_SETSCHEDPARAM" => {
            attr.priority = i32::from_le_bytes(
                m.read(a[1], 4)
                    .map_err(|_| Error::Invalid)?
                    .try_into()
                    .map_err(|_| Error::Invalid)?,
            );
            attrs.set(id, a[0], attr)?;
        }
        "ATTR_SETSCOPE" => {
            if a[1] != 2 {
                return Err(Error::Unsupported);
            }
        }
        "ATTR_GETSTACKSIZE" => write(m, a[1], &attr.stack_size.to_le_bytes())?,
        "ATTR_GETGUARDSIZE" => write(m, a[1], &attr.guard_size.to_le_bytes())?,
        "ATTR_GETSTACKADDR" => write(m, a[1], &attr.stack_address.to_le_bytes())?,
        "ATTR_GETSTACK" => {
            // Validate both outputs before publishing either value. These checked copies do
            // not promise atomicity against concurrent guest memory/protection changes.
            let address_bytes = read64(m, a[1])?.to_le_bytes();
            let size_bytes = read64(m, a[2])?.to_le_bytes();
            write(m, a[1], &address_bytes)?;
            write(m, a[2], &size_bytes)?;
            write(m, a[1], &attr.stack_address.to_le_bytes())?;
            write(m, a[2], &attr.stack_size.to_le_bytes())?;
        }
        "ATTR_GETDETACHSTATE" => write(m, a[1], &i32::from(attr.detached).to_le_bytes())?,
        "ATTR_GETINHERITSCHED" => write(m, a[1], &attr.inherit.to_le_bytes())?,
        "ATTR_GETSCHEDPOLICY" => write(m, a[1], &attr.policy.to_le_bytes())?,
        "ATTR_GETSCHEDPARAM" => write(m, a[1], &attr.priority.to_le_bytes())?,
        "ATTR_GETSCOPE" => write(m, a[1], &2i32.to_le_bytes())?,
        _ => return Err(Error::Unsupported),
    }
    Ok(0)
}
