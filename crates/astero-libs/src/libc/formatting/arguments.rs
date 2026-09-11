//! One checked GP/FP/overflow argument cursor for direct and explicit guest va_list calls.
use super::{Error, Result};
use astero_abi::layouts::{entry::CallFrame, varargs::VaList};
use astero_hle::calls::memory::GuestMemory;
pub struct Arguments<'a> {
    frame: Option<&'a CallFrame>,
    pub value: VaList,
    stack_index: usize,
}
impl<'a> Arguments<'a> {
    pub fn direct(f: &'a CallFrame, named: usize) -> Self {
        Self {
            frame: Some(f),
            value: VaList {
                gp_offset: (named * 8) as u32,
                fp_offset: 48,
                overflow_arg_area: f.stack_argument_address.unwrap_or(0),
                reg_save_area: 0,
            },
            stack_index: 0,
        }
    }
    pub fn list(m: &dyn GuestMemory, a: u64) -> Result<Self> {
        if a == 0 {
            return Err(Error::contract("null va_list"));
        }
        m.charge(24)?;
        m.validate(a, 24, false)?;
        let b = m.read(a, 24)?;
        let value = VaList::decode(b.try_into().map_err(|_| Error::contract("short va_list"))?)
            .ok_or_else(|| Error::contract("malformed va_list offsets/alignment"))?;
        Ok(Self {
            frame: None,
            value,
            stack_index: 0,
        })
    }
    fn word(m: &dyn GuestMemory, a: u64) -> Result<u64> {
        if a == 0 {
            return Err(Error::contract("null argument area"));
        }
        m.charge(8)?;
        m.validate(a, 8, false)?;
        Ok(u64::from_le_bytes(
            m.read(a, 8)?
                .try_into()
                .map_err(|_| Error::contract("short argument"))?,
        ))
    }
    fn overflow(&mut self, m: &dyn GuestMemory) -> Result<u64> {
        if let Some(f) = self.frame
            && f.stack_argument_address.is_none()
        {
            let v = *f
                .stack_arguments
                .get(self.stack_index)
                .ok_or_else(|| Error::contract("argument exceeds synthetic captured stack"))?;
            self.stack_index += 1;
            return Ok(v);
        }
        let a = self.value.overflow_arg_area;
        let next = a
            .checked_add(8)
            .ok_or_else(|| Error::contract("argument address overflow"))?;
        let v = Self::word(m, a)?;
        self.value.overflow_arg_area = next;
        Ok(v)
    }
    pub fn integer(&mut self, m: &dyn GuestMemory) -> Result<u64> {
        if self.value.gp_offset >= 48 {
            return self.overflow(m);
        }
        let offset = self.value.gp_offset;
        let v = if let Some(f) = self.frame {
            f.arguments[offset as usize / 8]
        } else {
            Self::word(
                m,
                self.value
                    .reg_save_area
                    .checked_add(offset as u64)
                    .ok_or_else(|| Error::contract("register area overflow"))?,
            )?
        };
        self.value.gp_offset += 8;
        Ok(v)
    }
    pub fn float(&mut self, m: &dyn GuestMemory) -> Result<f64> {
        if self.value.fp_offset >= 176 {
            return self.overflow(m).map(f64::from_bits);
        }
        let offset = self.value.fp_offset;
        let v = if let Some(f) = self.frame {
            u64::from_le_bytes(
                f.xmm[(offset as usize - 48) / 16][..8]
                    .try_into()
                    .expect("fixed lane"),
            )
        } else {
            Self::word(
                m,
                self.value
                    .reg_save_area
                    .checked_add(offset as u64)
                    .ok_or_else(|| Error::contract("FP area overflow"))?,
            )?
        };
        self.value.fp_offset += 16;
        Ok(f64::from_bits(v))
    }
}
