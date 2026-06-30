//! Per-tab view builders for the iocraft UI.
//!
//! Each view is a plain function that turns an owned `Frame` sub-struct plus the
//! frontend [`OverviewLayout`](crate::ui::layout) geometry into an
//! `AnyElement<'static>`. Phase 2A lands the splash and the Overview tab; the
//! CPU/GPU/Memory/SoC views follow in Phase 2B.

pub(crate) mod overview;
pub(crate) mod splash;

#[cfg(test)]
mod tests;
