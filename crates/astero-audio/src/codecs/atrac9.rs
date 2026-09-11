//! ATRAC9 configuration only, adapted from PS5Rust config/tables; no decoder.
use super::ajm::Error;
pub fn configuration(bytes: [u8; 4]) -> Result<[u32; 5], Error> {
    const RATES: [u32; 16] = [
        11025, 12000, 16000, 22050, 24000, 32000, 44100, 48000, 44100, 48000, 64000, 88200, 96000,
        128000, 176400, 192000,
    ];
    const POWERS: [u32; 16] = [6, 6, 7, 7, 7, 8, 8, 8, 6, 6, 7, 7, 7, 8, 8, 8];
    let v = u32::from_be_bytes(bytes);
    if v >> 24 != 0xfe || v & 0x10000 != 0 {
        return Err(Error::InvalidParameter);
    }
    let rate = ((v >> 20) & 15) as usize;
    let channel = ((v >> 17) & 7) as usize;
    let channels = *[1, 2, 2, 6, 8, 4]
        .get(channel)
        .ok_or(Error::InvalidParameter)?;
    let frame_bytes = ((v >> 5) & 2047) + 1;
    let superframe = (v >> 3) & 3;
    let samples = 1 << POWERS[rate];
    Ok([
        channels,
        RATES[rate],
        samples,
        samples << superframe,
        frame_bytes << superframe,
    ])
}
