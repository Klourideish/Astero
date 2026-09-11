//! Firmware establishes 80-byte output; Kyty/Prosper corroborate mask/angle views.
//! Topology values describe Astero's logical sink, never physical host capabilities.
pub const SPEAKER_INFO_SIZE: usize = 0x50;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpeakerInfo {
    bytes: [u8; SPEAKER_INFO_SIZE],
}
impl SpeakerInfo {
    /// Type byte 0, reserved 1..4, mask u32 at 4, flags u32 at 8,
    /// reserved 12..16, sixteen (azimuth:i16,elevation:i16) pairs at 16.
    /// Logical stereo is the supported null-sink topology, irrespective of input format.
    pub fn stereo_sink() -> Self {
        let mut bytes = [0; SPEAKER_INFO_SIZE];
        bytes[4..8].copy_from_slice(&3u32.to_le_bytes());
        bytes[16..18].copy_from_slice(&(-30i16).to_le_bytes());
        bytes[20..22].copy_from_slice(&30i16.to_le_bytes());
        Self { bytes }
    }
    pub fn bytes(&self) -> &[u8; SPEAKER_INFO_SIZE] {
        &self.bytes
    }
}
