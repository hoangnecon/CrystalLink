use serde::{Deserialize, Serialize};

pub const TILE_SIZE: usize = 32;
pub const MAX_UDP_PAYLOAD: usize = 1400;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Packet {
    Discovery {
        hostname: String,
    },
    FrameStart {
        frame_id: u32,
    },
    TileBatch {
        frame_id: u32,
        tiles: Vec<TileData>,
    },
    Cursor {
        x: u16,
        y: u16,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TileData {
    pub x: u16,
    pub y: u16,
    pub compression: CompressionType,
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CompressionType {
    Raw,
    Lz4,
    Jpeg,
}

// Helper to decompress tile data
pub fn decompress_tile(tile: &TileData) -> Vec<u8> {
    match tile.compression {
        CompressionType::Raw => tile.data.clone(),
        CompressionType::Lz4 => {
            lz4_flex::decompress_size_prepended(&tile.data).unwrap_or_else(|_| vec![0; TILE_SIZE * TILE_SIZE * 4])
        },
        CompressionType::Jpeg => {
            // Very basic JPEG decode (Receiver will need `image` crate support or custom)
            // For MVP, if we use `image` crate in common, we add weight.
            // Let's assume Receiver handles it. But for common lib helper:
            // This helper is mainly for debug. Real receiver might use hardware decoder.
            // We'll keep it unimplemented or simple panic for now if deps not added.
             vec![0; TILE_SIZE * TILE_SIZE * 4] // Placeholder
        }
    }
}
