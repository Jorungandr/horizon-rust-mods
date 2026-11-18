//! Validation probe: an empty hook set, nothing injected.
#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl mod_api::Hooks for Mod {}
