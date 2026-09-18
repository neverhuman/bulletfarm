//! Deterministic property-style tests over the wire canonical layer (T-3).
//!
//! Case `i` uses a documented deterministic stream, so every failure reports
//! a replayable seed and case index. Fixed negative controls keep the property
//! inventory from passing vacuously.

#[path = "properties/canonical.rs"]
mod canonical;
#[path = "properties/records.rs"]
mod records;
#[path = "properties/support.rs"]
mod support;
