//! Explicit SysV AMD64 guest va_list bytes, never a host C va_list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VaList {
    pub gp_offset: u32,
    pub fp_offset: u32,
    pub overflow_arg_area: u64,
    pub reg_save_area: u64,
}
impl VaList {
    pub fn decode(b: [u8; 24]) -> Option<Self> {
        let v = Self {
            gp_offset: u32::from_le_bytes(b[..4].try_into().ok()?),
            fp_offset: u32::from_le_bytes(b[4..8].try_into().ok()?),
            overflow_arg_area: u64::from_le_bytes(b[8..16].try_into().ok()?),
            reg_save_area: u64::from_le_bytes(b[16..24].try_into().ok()?),
        };
        if v.gp_offset > 48
            || !v.gp_offset.is_multiple_of(8)
            || v.fp_offset < 48
            || v.fp_offset > 176
            || !(v.fp_offset - 48).is_multiple_of(16)
            || !v.overflow_arg_area.is_multiple_of(8)
            || !v.reg_save_area.is_multiple_of(8)
        {
            None
        } else {
            Some(v)
        }
    }
}
