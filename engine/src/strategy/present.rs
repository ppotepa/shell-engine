/// Controls whether the virtual buffer is copied to the output buffer every frame
/// or skipped when the contents are unchanged.
pub trait VirtualPresenter: Send + Sync {
    fn should_skip(&self, hash: u64, last_hash: u64) -> bool;
}

/// Always presents the virtual buffer. Safe default.
pub struct AlwaysPresenter;

impl VirtualPresenter for AlwaysPresenter {
    #[inline]
    fn should_skip(&self, _hash: u64, _last_hash: u64) -> bool {
        false
    }
}

/// Skips the present step when the virtual buffer hash is unchanged.
/// Reduces terminal output for completely static scenes, but has a known bug:
/// when skipped, `fill()` is also skipped — which breaks dirty-region diff correctness.
/// Gate behind `--opt-present`. Do not combine with `--opt-diff`.
pub struct HashSkipPresenter;

impl VirtualPresenter for HashSkipPresenter {
    #[inline]
    fn should_skip(&self, hash: u64, last_hash: u64) -> bool {
        hash == last_hash
    }
}
