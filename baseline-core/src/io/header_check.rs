//! Shared "Check Header" summary used by Baseline, Calibration, and Flux

use super::science_packet::validate_science_packet;
use crate::observation::data_processor::get_date_time_from_hex_data;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

pub fn check_header_summary(
    path: &Path,
    delay_time_ms: i32,
    threshold: f64,
) -> std::io::Result<String> {
    const BUFFER_SIZE: usize = 16384;
    let file_len = std::fs::metadata(path)?.len();

    let mut file = std::fs::File::open(path)?;
    let mut head_buf = vec![0u8; BUFFER_SIZE.min(file_len as usize)];
    file.read_exact(&mut head_buf)?;
    let head_hex = String::from_utf8_lossy(&head_buf).to_string();

    let seek_pos = file_len.saturating_sub(BUFFER_SIZE as u64);
    file.seek(SeekFrom::Start(seek_pos))?;
    let mut tail_buf = Vec::new();
    file.read_to_end(&mut tail_buf)?;
    let tail_hex = String::from_utf8_lossy(&tail_buf).to_string();

    let mut out = String::new();
    out.push_str(&format!(
        "File Size: {:.2} MB\n",
        file_len as f64 / (1024.0 * 1024.0)
    ));
    out.push_str("--------------------------------------------------\n");
    out.push_str("[START OF FILE]\n");
    match parse_first_packet(&head_hex) {
        Some(bytes) => out.push_str(&parse_header_summary(&bytes)),
        None => out.push_str("Error: No complete science packet with a valid header and checksum at the start of file."),
    }
    out.push('\n');
    out.push_str("--------------------------------------------------\n");
    out.push_str("[END OF FILE]\n");
    match find_last_packet(&tail_hex) {
        Some(bytes) => out.push_str(&parse_header_summary(&bytes)),
        None => out.push_str("Warning: Could not identify a valid packet at the end of file.\n(File might be truncated or format is inconsistent)"),
    }
    out.push('\n');
    out.push_str("--------------------------------------------------\n");
    out.push_str(&format!("Delay Time: {delay_time_ms}\n"));
    out.push_str(&format!("Threshold: {threshold}\n"));

    Ok(out)
}

fn clean_hex(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
    if !hex.len().is_multiple_of(2) || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}

fn parse_first_packet(hex_content: &str) -> Option<Vec<u8>> {
    let clean = clean_hex(hex_content);
    clean
        .as_bytes()
        .windows(4)
        .enumerate()
        .filter(|(_, marker)| marker.eq_ignore_ascii_case(b"E225"))
        .find_map(|(offset, _)| packet_at(&clean, offset))
}

fn find_last_packet(hex_content: &str) -> Option<Vec<u8>> {
    let clean = clean_hex(hex_content);
    clean
        .as_bytes()
        .windows(4)
        .enumerate()
        .rev()
        .filter(|(_, marker)| marker.eq_ignore_ascii_case(b"E225"))
        .find_map(|(offset, _)| packet_at(&clean, offset))
}

fn packet_at(hex: &str, offset: usize) -> Option<Vec<u8>> {
    let length = u16::from_str_radix(hex.get(offset + 12..offset + 16)?, 16).ok()? as usize + 9;
    if !matches!(length, 256 | 2064) {
        return None;
    }
    let bytes = hex_to_bytes(hex.get(offset..offset + length * 2)?)?;
    let data_type = u16::from_be_bytes([bytes[14], bytes[15]]);
    validate_science_packet(&bytes, data_type).then_some(bytes)
}

fn parse_header_summary(data: &[u8]) -> String {
    if data.len() < 14 {
        return "Invalid/Incomplete Packet Data".to_string();
    }
    let hex_data = data
        .iter()
        .take(14)
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>();
    let Ok(dt) = get_date_time_from_hex_data(&hex_data) else {
        return "Invalid Packet Timecode".to_string();
    };

    format!(
        "Timestamp: {} (UTC)\nSync Code: {:02X} {:02X}\nSeq No:    {:02X} {:02X}",
        dt.format("%Y-%b-%d %H:%M:%S%.3f"),
        data[0],
        data[1],
        data[4],
        data[5]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packet(size: usize, mode: u8) -> String {
        let mut bytes = vec![0; size];
        bytes[..6].copy_from_slice(&[0xE2, 0x25, 0x08, 0xB0, 0xC0, 0]);
        bytes[6..8].copy_from_slice(&((size - 9) as u16).to_be_bytes());
        bytes[11] = 1;
        bytes[12] = 1;
        bytes[13] = 244;
        bytes[15] = mode;
        bytes[size - 2..].copy_from_slice(&(246 + mode as u16).to_be_bytes());
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    #[test]
    fn header_summary_reads_e225_both_packet_lengths_and_shared_timecode() {
        let first = packet(2064, 0xAE);
        let last = packet(256, 0xAD);
        let path =
            std::env::temp_dir().join(format!("match-header-check-{}.txt", std::process::id()));
        std::fs::write(&path, format!("{first}\n{last}\ne225")).unwrap();
        let summary = check_header_summary(&path, 0, 0.0).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(summary.matches("Sync Code: E2 25").count(), 2);
        assert_eq!(summary.matches("2024-Oct-01 00:00:01.500").count(), 2);
        assert!(!summary.contains("Warning:"));
    }

    #[test]
    fn ignores_false_sync_truncated_packets_and_bad_checksums() {
        let valid = packet(256, 0xAD);
        let mut corrupt = valid.clone();
        corrupt.replace_range(510..512, "00");
        assert!(parse_first_packet(&corrupt).is_none());
        assert!(find_last_packet(&corrupt).is_none());
        let data = format!("AA55E225{valid}{corrupt}e225");
        assert_eq!(parse_first_packet(&data), hex_to_bytes(&valid));
        assert_eq!(find_last_packet(&data), hex_to_bytes(&valid));
        assert!(parse_first_packet("E225💡").is_none());
    }
}
