//! Shared "Check Header" summary used by Baseline, Calibration, and Flux

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
        None => out.push_str("Error: Could not read start of file."),
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
    if !hex.len().is_multiple_of(2) {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}

fn parse_first_packet(hex_content: &str) -> Option<Vec<u8>> {
    if hex_content.is_empty() {
        return None;
    }
    let mut clean = clean_hex(hex_content);
    if clean.len() > 4128 {
        clean.truncate(4128);
    }
    if !clean.len().is_multiple_of(2) {
        clean.pop();
    }
    hex_to_bytes(&clean)
}

fn find_last_packet(hex_content: &str) -> Option<Vec<u8>> {
    if hex_content.is_empty() {
        return None;
    }
    let clean = clean_hex(hex_content);
    let upper = clean.to_ascii_uppercase();
    let last_index = upper.rfind("AA55")?;

    let expected_hex_length = 4128;
    if last_index + expected_hex_length <= clean.len() {
        hex_to_bytes(&clean[last_index..last_index + expected_hex_length])
    } else {
        let mut partial = clean[last_index..].to_string();
        if !partial.len().is_multiple_of(2) {
            partial.pop();
        }
        hex_to_bytes(&partial)
    }
}

fn parse_header_summary(data: &[u8]) -> String {
    if data.len() < 14 {
        return "Invalid/Incomplete Packet Data".to_string();
    }
    let seconds_part = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
    let milliseconds_part = ((data[13] as u16) << 8) | data[12] as u16;

    let total_ms = seconds_part as i64 * 1000 + milliseconds_part as i64;
    let dt =
        chrono::DateTime::from_timestamp(total_ms / 1000, ((total_ms % 1000) * 1_000_000) as u32)
            .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());

    format!(
        "Timestamp: {} (UTC)\nSync Code: {:02X} {:02X}\nSeq No:    {:02X} {:02X}",
        dt.format("%Y-%b-%d %H:%M:%S%.3f"),
        data[0],
        data[1],
        data[4],
        data[5]
    )
}
