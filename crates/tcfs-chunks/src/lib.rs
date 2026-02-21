//! tcfs-chunks: content-addressed chunking, BLAKE3 hashing, and seekable zstd compression
//!
//! # Overview
//! - `blake3`: deterministic file/slice hashing (content identity)
//! - `fastcdc`: content-defined chunking — stable boundaries even with inserts
//! - `seekable_zstd`: frame-based compression enabling random-access decompression
//! - `delta`: rsync rolling-hash delta sync (Phase 4 stub)

pub mod blake3;
pub mod delta;
pub mod fastcdc;
pub mod seekable_zstd;

// Convenience re-exports for the most common operations
pub use blake3::{Hash, hash_bytes, hash_file, hash_to_hex, hash_from_hex};
pub use fastcdc::{Chunk, ChunkSizes, chunk_data, chunk_file, chunk_slice};
pub use seekable_zstd::{SeekEntry, SeekableBlob, compress, decompress_all, decompress_range};
