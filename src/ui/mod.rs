//! Ui.
//!
//! The iocraft frontend: [`app_root::PumasApp`] drives the render loop, fed by
//! the backend `Frame` data plane. Tabs live under [`views`], shared widgets
//! under [`components`], and all pixel geometry is computed in [`layout`].

pub(crate) mod app_root;
pub(crate) mod components;
pub(crate) mod layout;
pub(crate) mod theme;
pub(crate) mod views;

#[cfg(test)]
mod snapshot;
