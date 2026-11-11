//! Data Table decoding for Flux mode.

use crate::io::science_packet::validate_science_hex_packet;
use crate::observation::data_processor::{get_date_time_from_hex_data, parse_line_header_fields};
use chrono::{DateTime, Utc};

const PARTICLE_TIME_START: usize = 16;
const PARTICLE_COUNTS_START: usize = 18;
/// Particle Counts L1-L7 (Byte 18-31), one per layer.
pub const PARTICLE_COUNTS_LAYERS: usize = 7;
const PARTICLE_INFO_START: usize = 32;
/// Particle Info (Byte 32-2031): 1000 decoded decimal values, 2 bytes each.
pub const PARTICLE_INFO_COUNT: usize = 1000;

const RESERVED_START: usize = 2032;
const RESERVED_LEN: usize = 30; // Byte 2032-2061
const CHECKSUM_START: usize = 2062;
const CHECKSUM_LEN: usize = 2; // Byte 2062-2063

/// One decoded raw line for the Flux mode Data Table tab.
#[derive(Debug, Clone, Default)]
pub struct FluxLineResult {
    pub packet_sync: String,
    pub package_id: i32,
    pub packet_sequence: i32,
    pub packet_data_length: i32,
    pub time: DateTime<Utc>,
    pub data_type: String,
    pub particle_time: i32,
    /// Particle Counts L1-L7 (Byte 18-31), in that order.
    pub particle_counts: Vec<i32>,
    /// Particle Info (Byte 32-2031): `PARTICLE_INFO_COUNT` decoded decimal values.
    pub particle_info: Vec<i32>,
    pub reserved_hex: String,
    pub checksum_hex: String,
}

fn byte(hex_data: &[String], i: usize) -> i32 {
    hex_data
        .get(i)
        .and_then(|h| i32::from_str_radix(h, 16).ok())
        .unwrap_or(0)
}

fn dec_pair(hex_data: &[String], i: usize) -> i32 {
    (byte(hex_data, i) << 8) + byte(hex_data, i + 1)
}

fn hex_span(hex_data: &[String], start: usize, len: usize) -> String {
    (start..start + len)
        .map(|i| hex_data.get(i).cloned().unwrap_or_else(|| "00".to_string()))
        .collect()
}

/// Decodes one validated flux packet into a Data Table row.
pub fn parse_flux_line(hex_data: &[String]) -> Option<FluxLineResult> {
    if !validate_science_hex_packet(hex_data, 0x00AC) {
        return None;
    }
    let header = parse_line_header_fields(hex_data)?;
    let time = get_date_time_from_hex_data(hex_data).ok()?;
    let particle_time = dec_pair(hex_data, PARTICLE_TIME_START);
    let particle_counts = (0..PARTICLE_COUNTS_LAYERS)
        .map(|i| dec_pair(hex_data, PARTICLE_COUNTS_START + i * 2))
        .collect();
    let particle_info = (0..PARTICLE_INFO_COUNT)
        .map(|i| dec_pair(hex_data, PARTICLE_INFO_START + i * 2))
        .collect();

    Some(FluxLineResult {
        packet_sync: header.packet_sync,
        package_id: header.package_id,
        packet_sequence: header.packet_sequence,
        packet_data_length: header.packet_data_length,
        time,
        data_type: header.data_type,
        particle_time,
        particle_counts,
        particle_info,
        reserved_hex: hex_span(hex_data, RESERVED_START, RESERVED_LEN),
        checksum_hex: hex_span(hex_data, CHECKSUM_START, CHECKSUM_LEN),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flux::processing::{
        get_date_time_from_hex, process_header_internal, FluxAccumulator,
    };

    fn packet() -> Vec<u8> {
        let mut bytes = vec![0; 2064];
        bytes[..8].copy_from_slice(&[0xE2, 0x25, 0x08, 0xB0, 0xC0, 0, 0x08, 0x07]);
        bytes[8..14].copy_from_slice(&[0, 0, 0, 1, 1, 244]);
        bytes[15] = 0xAC;
        bytes[16..18].copy_from_slice(&1500u16.to_be_bytes());
        for layer in 0..7 {
            bytes[19 + layer * 2] = 101 + layer as u8;
        }
        bytes[32..34].copy_from_slice(&[0xEA, 0xBC]);
        bytes[2032] = 0xAB;
        bytes[2061] = 0xCD;
        // Time 246 + mode 172 + duration 225 + counters 728 + event 422 + reserved 376.
        bytes[2062..].copy_from_slice(&2169u16.to_be_bytes());
        bytes
    }

    fn hex(bytes: &[u8]) -> Vec<String> {
        bytes.iter().map(|byte| format!("{byte:02X}")).collect()
    }

    #[test]
    fn decodes_duration_counters_layer_time_bits_and_reserved_span() {
        let data = hex(&packet());
        let row = parse_flux_line(&data).unwrap();
        assert_eq!(row.particle_time, 1500);
        assert_eq!(row.particle_counts, vec![101, 102, 103, 104, 105, 106, 107]);
        assert_eq!(row.particle_info.len(), 1000);
        assert_eq!(row.reserved_hex.len(), 30 * 2);
        assert!(row.reserved_hex.starts_with("AB"));
        assert!(row.reserved_hex.ends_with("CD"));
        let mut accumulator = FluxAccumulator::default();
        assert!(accumulator.process_flux_observation(&data.concat()));
        assert_eq!(accumulator.seconds_part_list, vec![1.5]);
        assert_eq!(accumulator.particle_counting_lists[6], vec![107.0]);
        assert_eq!(accumulator.particle_layer_list[0][0], 7.0);
        assert_eq!(accumulator.particle_offset_time_list[0][0], 2748.0);
        assert_eq!(row.time.to_rfc3339(), "2024-10-01T00:00:01.500+00:00");
        assert_eq!(get_date_time_from_hex(&data.concat()), row.time);
        let header = process_header_internal(&data).unwrap();
        assert_eq!(header.timestamp, "Timestamp: 2024-Oct-01 00:00:01.500");
        assert!(header.checksum_matches);
    }

    #[test]
    fn rejects_bad_checksum_and_other_modes_without_accumulating() {
        let mut corrupt = packet();
        corrupt[2063] ^= 1;
        let mut wrong_mode = packet();
        wrong_mode[15] = 0xAE;
        wrong_mode[2062..].copy_from_slice(&2171u16.to_be_bytes());
        let mut accumulator = FluxAccumulator::default();
        for bytes in [corrupt, wrong_mode] {
            let data = hex(&bytes);
            assert!(parse_flux_line(&data).is_none());
            assert!(!accumulator.process_flux_observation(&data.concat()));
        }
        let mut malformed = hex(&packet()).concat();
        malformed.replace_range(10..12, "+0");
        assert!(!accumulator.process_flux_observation(&malformed));
        assert!(accumulator.all_results.is_empty());
        assert!(accumulator.seconds_part_list.is_empty());
    }
}
