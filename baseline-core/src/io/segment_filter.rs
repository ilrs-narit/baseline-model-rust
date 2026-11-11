//! Extract complete, validated MATCH science packets from hex-text files.
use super::science_packet::validate_science_packet;
use std::fs;
use std::path::Path;

pub fn filter_e225_segments_from_files(files: &[impl AsRef<Path>], chunk_hex_len: usize) -> Result<Vec<String>, String> {
    if !matches!(chunk_hex_len, 512 | 4128) {
        return Err("Science packet length must be 512 or 4128 hex characters".to_string());
    }
    let mut filtered_segments = Vec::new();

    for file in files {
        let content = fs::read_to_string(file.as_ref()).map_err(|e| e.to_string())?;
        let cleaned: String = content.chars().filter(|c| !c.is_whitespace()).collect();
        let source = cleaned.as_bytes();
        let mut offset = 0;
        while offset + chunk_hex_len <= source.len() {
            if !source[offset..offset + 4].eq_ignore_ascii_case(b"E225") {
                offset += 1;
                continue;
            }
            let candidate = &source[offset..offset + chunk_hex_len];
            let bytes = candidate.chunks_exact(2).map(|pair| {
                Some(((pair[0] as char).to_digit(16)? * 16
                    + (pair[1] as char).to_digit(16)?) as u8)
            }).collect::<Option<Vec<_>>>();
            if let Some(bytes) = bytes {
                let mode = u16::from_be_bytes([bytes[14], bytes[15]]);
                if validate_science_packet(&bytes, mode) {
                    filtered_segments.push(String::from_utf8(candidate.to_vec()).expect("validated ASCII hex"));
                    offset += chunk_hex_len;
                    continue;
                }
            }
            offset += 1;
        }
    }
    Ok(filtered_segments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovers_valid_packets_after_noise_corruption_and_truncation() {
        for mode in [0xAD, 0xAE] {
            let byte_len = if mode == 0xAD { 256 } else { 2064 };
            let mut packet = vec![0; byte_len];
            packet[..8].copy_from_slice(&[
                0xE2, 0x25, 0x08, 0xB0, 0xC0, 0,
                if mode == 0xAD { 0 } else { 0x08 },
                if mode == 0xAD { 0xF7 } else { 0x07 },
            ]);
            packet[15] = mode;
            let reserved = if mode == 0xAD { 186 } else { 1938 };
            let embedded_header = packet[..16].to_vec();
            packet[reserved + 4..reserved + 20].copy_from_slice(&embedded_header);
            let checksum = packet[8..byte_len - 2].iter().map(|&byte| byte as u16).sum::<u16>();
            packet[byte_len - 2..].copy_from_slice(&checksum.to_be_bytes());
            let valid = packet.iter().map(|byte| format!("{byte:02X}")).collect::<String>();
            let mut bad_checksum = packet.clone();
            bad_checksum[byte_len - 1] ^= 1;
            let bad = bad_checksum.iter().map(|byte| format!("{byte:02X}")).collect::<String>();
            let mut malformed = valid.clone();
            malformed.insert_str(100, "GG");
            let content = format!("E2250000{bad}{}{malformed}ไทย{}\n{valid}", &valid[..100], valid.to_ascii_lowercase());
            let path = std::env::temp_dir().join(format!("match-segment-filter-{}-{mode}.txt", std::process::id()));
            fs::write(&path, content).unwrap();
            let filtered = filter_e225_segments_from_files(&[&path], byte_len * 2);
            fs::remove_file(&path).unwrap();
            assert_eq!(filtered.unwrap(), vec![valid.to_ascii_lowercase(), valid]);
        }
    }

    #[test]
    fn rejects_unsupported_chunk_lengths() {
        let files: [&Path; 0] = [];
        assert!(filter_e225_segments_from_files(&files, 0).is_err());
        assert!(filter_e225_segments_from_files(&files, 32).is_err());
    }
}
