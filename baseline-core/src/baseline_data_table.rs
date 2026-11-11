//! Data Table decoding for Baseline mode

use crate::io::science_packet::validate_science_hex_packet;
use crate::observation::data_processor::{get_date_time_from_hex_data, parse_line_header_fields};
use chrono::{DateTime, Utc};

const L1L2_START: usize = 18;
const L6L7_START: usize = 978;
/// One layer's channel block within a sample, in bytes (16 channels x 2 bytes).
pub const BLOCK_LEN: usize = 32;
/// Decoded values per block (one per channel).
const VALUES_PER_BLOCK: usize = BLOCK_LEN / 2;
/// Byte stride between consecutive samples: one block per layer, two layers
/// per section (e.g. L1 then L2).
const SAMPLE_STRIDE: usize = BLOCK_LEN * 2;
pub const BLOCKS_PER_LAYER: usize = 15;

pub const L1_OFFSET: usize = L1L2_START;
pub const L2_OFFSET: usize = L1L2_START + BLOCK_LEN;
pub const L6_OFFSET: usize = L6L7_START;
pub const L7_OFFSET: usize = L6L7_START + BLOCK_LEN;

/// Byte offset of the `index`'th (0-based) sample's block for one layer,
/// given that layer's `L1_OFFSET`/`L2_OFFSET`/`L6_OFFSET`/`L7_OFFSET`.
pub const fn sample_block_offset(layer_base: usize, index: usize) -> usize {
    layer_base + SAMPLE_STRIDE * index
}

const RESERVED_START: usize = 1938;
const RESERVED_LEN: usize = 124; // Byte 1938-2061
const CHECKSUM_START: usize = 2062;
const CHECKSUM_LEN: usize = 2; // Byte 2062-2063

/// One decoded raw line for the Baseline mode Data Table tab.
#[derive(Debug, Clone, Default)]
pub struct BaselineLineResult {
    pub packet_sync: String,
    pub package_id: i32,
    pub packet_sequence: i32,
    pub packet_data_length: i32,
    pub time: DateTime<Utc>,
    pub data_type: String,
    pub sample_index: i32,
    /// Blocks of 16 decoded per-channel values, one `Vec` per sample block,
    /// for each layer.
    pub l1_blocks: Vec<Vec<i32>>,
    pub l2_blocks: Vec<Vec<i32>>,
    pub l6_blocks: Vec<Vec<i32>>,
    pub l7_blocks: Vec<Vec<i32>>,
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

/// Decodes one 32-byte channel block into its `VALUES_PER_BLOCK` (16)
/// per-channel values.
fn block_values(hex_data: &[String], start: usize) -> Vec<i32> {
    (0..VALUES_PER_BLOCK)
        .map(|i| dec_pair(hex_data, start + i * 2))
        .collect()
}

/// `BLOCKS_PER_LAYER` sample blocks for one layer, per `sample_block_offset`.
fn layer_blocks(hex_data: &[String], layer_base: usize) -> Vec<Vec<i32>> {
    (0..BLOCKS_PER_LAYER)
        .map(|index| block_values(hex_data, sample_block_offset(layer_base, index)))
        .collect()
}

/// Decodes one raw baseline line into a Data Table row, per `Baseline.txt`.
pub fn parse_baseline_line(hex_data: &[String]) -> Option<BaselineLineResult> {
    if !validate_science_hex_packet(hex_data, 0x00AE) {
        return None;
    }
    let header = parse_line_header_fields(hex_data)?;
    let time = get_date_time_from_hex_data(hex_data).ok()?;
    let sample_index = dec_pair(hex_data, 16);

    Some(BaselineLineResult {
        packet_sync: header.packet_sync,
        package_id: header.package_id,
        packet_sequence: header.packet_sequence,
        packet_data_length: header.packet_data_length,
        time,
        data_type: header.data_type,
        sample_index,
        l1_blocks: layer_blocks(hex_data, L1_OFFSET),
        l2_blocks: layer_blocks(hex_data, L2_OFFSET),
        l6_blocks: layer_blocks(hex_data, L6_OFFSET),
        l7_blocks: layer_blocks(hex_data, L7_OFFSET),
        reserved_hex: hex_span(hex_data, RESERVED_START, RESERVED_LEN),
        checksum_hex: hex_span(hex_data, CHECKSUM_START, CHECKSUM_LEN),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packet() -> Vec<u8> {
        let mut bytes = vec![0; 2064];
        bytes[..8].copy_from_slice(&[0xE2, 0x25, 0x08, 0xB0, 0xC0, 0, 0x08, 0x07]);
        bytes[15] = 0xAE;
        bytes[17] = 1;
        for block in 0..60 {
            for channel in 0..16 {
                bytes[19 + block * 32 + channel * 2] = block as u8 + 1;
            }
        }
        bytes[1938] = 0xAB;
        bytes[2061] = 0xCD;
        // 16 channels * sum(1..=60) + mode 174 + sample 1 + reserved 171+205.
        bytes[2062..].copy_from_slice(&29831u16.to_be_bytes());
        bytes
    }

    fn hex(bytes: &[u8]) -> Vec<String> {
        bytes.iter().map(|byte| format!("{byte:02X}")).collect()
    }

    #[test]
    fn decodes_fifteen_samples_and_preserves_all_reserved_bytes() {
        let row = parse_baseline_line(&hex(&packet())).unwrap();
        assert_eq!(row.sample_index, 1);
        assert_eq!(row.l1_blocks.len(), 15);
        assert_eq!(row.l1_blocks[14], vec![29; 16]);
        assert_eq!(row.l2_blocks[14], vec![30; 16]);
        assert_eq!(row.l6_blocks[0], vec![31; 16]);
        assert_eq!(row.l7_blocks[14], vec![60; 16]);
        assert_eq!(row.reserved_hex.len(), 124 * 2);
        assert!(row.reserved_hex.starts_with("AB"));
        assert!(row.reserved_hex.ends_with("CD"));
    }

    #[test]
    fn rejects_corrupt_checksum_and_valid_packet_of_another_mode() {
        let mut corrupt = packet();
        corrupt[2063] ^= 1;
        assert!(parse_baseline_line(&hex(&corrupt)).is_none());
        let mut wrong_mode = packet();
        wrong_mode[15] = 0xAC;
        wrong_mode[2062..].copy_from_slice(&29829u16.to_be_bytes());
        assert!(parse_baseline_line(&hex(&wrong_mode)).is_none());
    }

    #[test]
    fn stream_skips_invalid_candidates_and_recovers_the_next_valid_packet() {
        let valid = packet();
        let mut corrupt = valid.clone();
        corrupt[2063] ^= 1;
        let mut wrong_mode = valid.clone();
        wrong_mode[15] = 0xAC;
        wrong_mode[2062..].copy_from_slice(&29829u16.to_be_bytes());
        let mut inserted_nonhex = hex(&valid).concat();
        inserted_nonhex.insert_str(64, "GG");
        let mut replaced_nonhex = hex(&valid).concat();
        replaced_nonhex.replace_range(10..12, "GG");
        let path = std::env::temp_dir().join(format!("match-baseline-{}.txt", std::process::id()));
        let content = format!("E2250000{}{}{}{}{}", hex(&corrupt).concat(), hex(&wrong_mode).concat(), inserted_nonhex, replaced_nonhex, hex(&valid).join(" \n"));
        std::fs::write(&path, content).unwrap();
        let result = crate::io::baseline_file_service::process_file_stream(&path, None);
        std::fs::remove_file(&path).unwrap();
        let samples = result.unwrap();
        assert_eq!(samples.len(), 15);
        assert_eq!(samples[0].l6, [31.0; 16]);
        assert_eq!(samples[14].l7, [60.0; 16]);
    }
}
