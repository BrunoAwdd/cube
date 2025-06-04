use sha2::{Sha256, Digest};

/// Computes the SHA-256 hash of the given binary data and returns it as a hexadecimal string.
///
/// # Arguments
/// * `data` - A byte slice containing the data to hash.
///
/// # Returns
/// A `String` containing the hexadecimal representation of the SHA-256 hash.
///
/// # Example
/// ```
/// let hash = compute_hash(b"hello world");
/// println!("{}", hash);
/// ```
pub fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}