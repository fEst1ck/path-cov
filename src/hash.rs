use byteorder::{ByteOrder, LittleEndian};
use sha2::{Sha256, Digest, digest};

pub fn hash_path(path: &[u64]) -> digest::Output<Sha256> {
    let mut bytes: Vec<u8> = vec![0; path.len() * std::mem::size_of::<u64>()];
    LittleEndian::write_u64_into(path, &mut bytes);
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize()
}