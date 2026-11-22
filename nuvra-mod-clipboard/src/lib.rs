// nuvra-mod-clipboard — clipboard history module.
// R-13: No unsafe.
// D-DAT-01/02/03: AES-256-GCM storage, limits, exclusions.
#![forbid(unsafe_code)]

pub mod exclusions;
pub mod module;
pub mod state;
