//! Data Table decoding for Calibration mode

use crate::io::science_packet::validate_science_hex_packet;
use crate::observation::data_processor::{get_date_time_from_hex_data, parse_line_header_fields};
use chrono::{DateTime, Utc};

const VOLTAGE_STEPS: usize = 11;
const BLOCK_START: usize = 18;
const BLOCK_LEN: usize = 32;
const BLOCK_COUNT: usize = VOLTAGE_STEPS * 4;
const RESERVED_START: usize = 1426;
const RESERVED_LEN: usize = 636; // Byte 1426-2061
const CHECKSUM_START: usize = 2062;
const CHECKSUM_LEN: usize = 2; // Byte 2062-2063

/// One decoded 32-byte block column (e.g. "0.05V L1"): 16 values, 2 bytes
/// each, decoded to decimal.
pub struct CalibrationBlockField {
    pub label: String,
    pub byte_start: usize,
}

/// Byte 18-1425: L1/L2 at all 11 voltage steps, followed by L6/L7.
pub fn calibration_block_fields() -> Vec<CalibrationBlockField> {
    let mut fields = Vec::with_capacity(BLOCK_COUNT);
    for layers in [["L1", "L2"], ["L6", "L7"]] {
        for step in 0..VOLTAGE_STEPS {
            for layer in layers {
                fields.push(CalibrationBlockField {
                    label: format!("{:.2}V {}", step as f64 * 0.05, layer),
                    byte_start: BLOCK_START + fields.len() * BLOCK_LEN,
                });
            }
        }
    }
    fields
}

/// One decoded raw line for the Calibration mode Data Table tab.
#[derive(Debug, Clone, Default)]
pub struct CalibrationLineResult {
    pub packet_sync: String,
    pub package_id: i32,
    pub packet_sequence: i32,
    pub packet_data_length: i32,
    pub time: DateTime<Utc>,
    pub data_type: String,
    pub sample_index: i32,
    /// 44 columns (11 voltage steps x 4 layers, matching `calibration_block_fields`),
    /// each holding 16 decoded decimal values.
    pub blocks: Vec<Vec<i32>>,
    pub reserved_hex: String,
    pub checksum_hex: String,
}

fn byte(hex_data: &[String], i: usize) -> i32 {
    hex_data.get(i).and_then(|h| i32::from_str_radix(h, 16).ok()).unwrap_or(0)
}

fn dec_pair(hex_data: &[String], i: usize) -> i32 {
    (byte(hex_data, i) << 8) + byte(hex_data, i + 1)
}

fn hex_span(hex_data: &[String], start: usize, len: usize) -> String {
    (start..start + len).map(|i| hex_data.get(i).cloned().unwrap_or_else(|| "00".to_string())).collect()
}

/// Decodes one 32-byte block into 16 decimal values, 2 bytes each.
fn decode_block(hex_data: &[String], byte_start: usize) -> Vec<i32> {
    (0..BLOCK_LEN / 2).map(|j| dec_pair(hex_data, byte_start + j * 2)).collect()
}

/// Decodes one raw calibration line into a Data Table row.
pub fn parse_calibration_line(hex_data: &[String]) -> Option<CalibrationLineResult> {
    if !validate_science_hex_packet(hex_data, 0x00AB) {
        return None;
    }
    let header = parse_line_header_fields(hex_data)?;
    let time = get_date_time_from_hex_data(hex_data).ok()?;
    let sample_index = dec_pair(hex_data, 16);

    let blocks = (0..BLOCK_COUNT)
        .map(|block| decode_block(hex_data, BLOCK_START + block * BLOCK_LEN))
        .collect();

    Some(CalibrationLineResult {
        packet_sync: header.packet_sync,
        package_id: header.package_id,
        packet_sequence: header.packet_sequence,
        packet_data_length: header.packet_data_length,
        time,
        data_type: header.data_type,
        sample_index,
        blocks,
        reserved_hex: hex_span(hex_data, RESERVED_START, RESERVED_LEN),
        checksum_hex: hex_span(hex_data, CHECKSUM_START, CHECKSUM_LEN),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calibration::CalibrationAccumulator;

    fn calibration_packet() -> Vec<String> {
        let mut bytes = vec![0u8; 2064];
        for (offset, value) in [
            (0, 0xE225u16), (2, 0x08B0), (4, 0xC000), (6, 0x0807),
            (14, 0x00AB), (16, 1),
            // MATCH table 3: L1/L2 voltage sweep, then L6/L7 voltage sweep.
            (18, 101), (50, 201), (82, 102), (114, 202),
            (658, 111), (690, 211), (722, 601), (754, 701),
            (786, 602), (818, 702), (1362, 611), (1394, 711),
        ] {
            bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
        }
        bytes[1426..2062].fill(0xA5);
        let sum = bytes[8..2062].iter().fold(0u16, |sum, &byte| sum.wrapping_add(byte as u16));
        bytes[2062..].copy_from_slice(&sum.to_be_bytes());
        bytes.iter().map(|byte| format!("{byte:02X}")).collect()
    }

    #[test]
    fn decodes_specification_voltage_sweeps_without_mixing_layers() {
        let packet = calibration_packet();
        let fields = calibration_block_fields();
        for (index, label, offset) in [
            (0, "0.00V L1", 18), (1, "0.00V L2", 50),
            (2, "0.05V L1", 82), (21, "0.50V L2", 690),
            (22, "0.00V L6", 722), (23, "0.00V L7", 754),
            (42, "0.50V L6", 1362), (43, "0.50V L7", 1394),
        ] {
            assert_eq!(fields[index].label, label);
            assert_eq!(fields[index].byte_start, offset);
        }

        let decoded = parse_calibration_line(&packet).unwrap();
        assert_eq!(decoded.blocks.len(), 44);
        assert!(decoded.blocks.iter().all(|block| block.len() == 16));
        assert_eq!(decoded.blocks[2][0], 102);
        assert_eq!(decoded.blocks[22][0], 601);
        assert_eq!(decoded.blocks[43][0], 711);
        assert_eq!(decoded.reserved_hex, "A5".repeat(636));

        let mut accumulator = CalibrationAccumulator::default();
        accumulator.process_calibration(&packet);
        assert_eq!(accumulator.l1_columns[0][..2], [101.0, 102.0]);
        assert_eq!(accumulator.l2_columns[0][10], 211.0);
        assert_eq!(accumulator.l6_columns[0][..2], [601.0, 602.0]);
        assert_eq!(accumulator.l7_columns[0][10], 711.0);
    }

    #[test]
    fn rejects_bad_checksum_wrong_mode_and_incomplete_packets() {
        let packet = calibration_packet();
        let mut bad_checksum = packet.clone();
        bad_checksum[18] = "FF".to_string();
        let mut wrong_mode = packet.clone();
        wrong_mode[15] = "AC".to_string();
        let sum = wrong_mode[8..2062].iter().map(|byte| u16::from_str_radix(byte, 16).unwrap()).fold(0u16, u16::wrapping_add);
        wrong_mode[2062] = format!("{:02X}", sum >> 8);
        wrong_mode[2063] = format!("{:02X}", sum & 0xFF);
        for invalid in [bad_checksum, wrong_mode, packet[..2063].to_vec()] {
            assert!(parse_calibration_line(&invalid).is_none());
            let mut accumulator = CalibrationAccumulator::default();
            accumulator.process_calibration(&invalid);
            assert!(accumulator.l1_columns.iter().all(Vec::is_empty));
            assert!(accumulator.l7_columns.iter().all(Vec::is_empty));
        }
    }
}
