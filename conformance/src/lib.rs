//! Conformance test harness — validates workspace builds and owned fixtures.
//!
//! See `conformance/README.md` for the full strategy.

#[cfg(test)]
mod config_tests;
#[cfg(test)]
mod e2e_tests;
#[cfg(test)]
mod http_blackbox_tests;
#[cfg(test)]
mod rtk_tests;
#[cfg(test)]
mod workspace_tests;
