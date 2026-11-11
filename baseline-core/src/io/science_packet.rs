//! Validation of complete MATCH science packets (PDF sections 1.2 and 2.2).

pub fn validate_science_packet(bytes: &[u8], expected_type: u16) -> bool {
    let packet_len = match expected_type {
        0x00AD => 256,
        0x00AB | 0x00AC | 0x00AE => 2064,
        _ => return false,
    };
    if bytes.len() != packet_len {
        return false;
    }
    let pair = |offset| u16::from_be_bytes([bytes[offset], bytes[offset + 1]]);
    // The PDF gives conflicting APIDs; only its unambiguous header flags are checked.
    if pair(0) != 0xE225
        || pair(2) & 0xF800 != 0x0800
        || pair(4) & 0xC000 != 0xC000
        || usize::from(pair(6)) + 9 != packet_len
        || pair(14) != expected_type
    {
        return false;
    }
    let checksum_start = packet_len - 2;
    let checksum = bytes[8..checksum_start]
        .iter()
        .fold(0u16, |sum, &byte| sum.wrapping_add(u16::from(byte)));
    checksum == pair(checksum_start)
}

pub fn validate_science_hex_packet(hex_data: &[String], expected_type: u16) -> bool {
    let bytes = hex_data
        .iter()
        .map(|hex| {
            if hex.len() != 2 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return None;
            }
            u8::from_str_radix(hex, 16).ok()
        })
        .collect::<Option<Vec<_>>>();
    bytes.is_some_and(|bytes| validate_science_packet(&bytes, expected_type))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packet(data_type: u16) -> Vec<u8> {
        let len = if data_type == 0xAD { 256 } else { 2064 };
        let mut bytes = vec![0; len];
        bytes[..8].copy_from_slice(&[
            0xE2,
            0x25,
            0x08,
            0xB0,
            0xC0,
            0x00,
            if len == 256 { 0x00 } else { 0x08 },
            if len == 256 { 0xF7 } else { 0x07 },
        ]);
        bytes[11] = 1;
        bytes[12] = 1;
        bytes[13] = 244;
        bytes[15] = data_type as u8;
        // Time bytes sum to 246; the only nonzero payload byte is the mode marker.
        bytes[len - 2..].copy_from_slice(&(246 + data_type).to_be_bytes());
        bytes
    }

    #[test]
    fn accepts_both_science_lengths_and_all_four_modes() {
        for data_type in [0xAB, 0xAC, 0xAD, 0xAE] {
            assert!(validate_science_packet(&packet(data_type), data_type));
        }
    }

    #[test]
    fn rejects_corruption_wrong_mode_and_inconsistent_headers() {
        let valid = packet(0xAB);
        for (offset, value) in [
            (0, 0xAA),
            (2, 0x28),
            (2, 0x18),
            (2, 0x00),
            (4, 0x80),
            (6, 0),
            (15, 0xAC),
            (16, 1),
            (2063, 0),
        ] {
            let mut corrupt = valid.clone();
            corrupt[offset] = value;
            assert!(!validate_science_packet(&corrupt, 0xAB), "offset {offset}");
        }
        assert!(!validate_science_packet(&valid, 0xAC));
        assert!(!validate_science_packet(&valid[..2063], 0xAB));
        let mut extra = valid.clone();
        extra.push(0);
        assert!(!validate_science_packet(&extra, 0xAB));
        assert!(!validate_science_packet(&[], 0xAB));
        assert!(!validate_science_packet(&valid, 0));
    }

    #[test]
    fn checksum_wraps_and_excludes_primary_header() {
        let mut bytes = packet(0xAE);
        bytes[16..2062].fill(0xFF);
        // 246 time + 174 marker + 2046*255 data = 522150; lower 16 bits = 63398.
        bytes[2062..].copy_from_slice(&63398u16.to_be_bytes());
        bytes[3] = 0xF7;
        bytes[4] = 0xFF;
        bytes[5] = 0xFF;
        assert!(validate_science_packet(&bytes, 0xAE));
    }

    #[test]
    fn hex_adapter_rejects_malformed_byte_tokens() {
        let mut hex = packet(0xAD)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<Vec<_>>();
        assert!(validate_science_hex_packet(&hex, 0xAD));
        for malformed in ["GG", "000", "+0", ""] {
            hex[16] = malformed.to_string();
            assert!(!validate_science_hex_packet(&hex, 0xAD));
        }
    }
}
