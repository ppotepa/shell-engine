/// Deterministic per-pixel hash — changes every frame as `frame` (derived from progress) advances.
pub fn crt_hash(x: u16, y: u16, frame: u32) -> u32 {
    let v = (x as u32)
        .wrapping_mul(2_654_435_761)
        .wrapping_add((y as u32).wrapping_mul(2_246_822_519))
        .wrapping_add(frame.wrapping_mul(1_013_904_223));
    v ^ (v >> 16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crt_hash_is_deterministic() {
        let a = crt_hash(10, 20, 3);
        let b = crt_hash(10, 20, 3);
        assert_eq!(a, b);
    }

    #[test]
    fn crt_hash_changes_with_frame() {
        assert_ne!(crt_hash(10, 20, 3), crt_hash(10, 20, 4));
    }
}
