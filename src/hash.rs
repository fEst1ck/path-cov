use sha2::{Sha256, Digest};

use crate::extern_cfg::BlockID;

pub fn hash_path(path: &[BlockID]) -> String {
    let mut hasher = Sha256::new();
    for &value in path {
        hasher.update(value.to_ne_bytes());
    }
    let result = hasher.finalize();
    hex::encode(result)
}
