//! Fire control of simple cannons.

/// Denotes if the cannon is firing, boolean value.
pub const REG_CANNON_FIRING: u32 = 0;

/// Denotes if the cannon is ready to fire, boolean value.
pub const REG_CANNON_READY: u32 = 1;

/// Provides the reload time in seconds, float value.
pub const REG_CANNON_RELOAD_TIME: u32 = 2;