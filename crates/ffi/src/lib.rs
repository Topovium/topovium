// SPDX-License-Identifier: GPL-3.0-or-later
//! The C ABI the Swift and Kotlin shells call.
//!
//! One of only two places in the workspace where `unsafe` is permitted, because a
//! foreign function interface is unsafe by construction. Three rules keep the blast
//! radius small:
//!
//! 1. **No Rust type crosses the boundary.** Only C-representable values and opaque
//!    pointers. Adding a field to a Rust struct must never change the ABI.
//! 2. **No panic crosses the boundary.** Unwinding into Swift or Kotlin is undefined
//!    behaviour, so every entry point catches and converts to a status code.
//! 3. **Ownership is explicit.** Anything Rust allocates, Rust frees, through a
//!    matching `topovium_*_free`.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::ffi::{CString, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};
use topovium_platform_api::ThermalState;

/// Result of an FFI call. Zero is success; every other value is a specific failure.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopoviumStatus {
    Ok = 0,
    /// A panic was caught at the boundary. The shell should report and recover rather
    /// than continuing with state it cannot reason about.
    InternalError = 1,
    NullArgument = 2,
    InvalidArgument = 3,
}

/// Thermal state as the shells report it, mirroring `ThermalState` in a C-safe form.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopoviumThermalState {
    Nominal = 0,
    Fair = 1,
    Serious = 2,
    Critical = 3,
}

impl From<TopoviumThermalState> for ThermalState {
    fn from(value: TopoviumThermalState) -> Self {
        match value {
            TopoviumThermalState::Nominal => Self::Nominal,
            TopoviumThermalState::Fair => Self::Fair,
            TopoviumThermalState::Serious => Self::Serious,
            TopoviumThermalState::Critical => Self::Critical,
        }
    }
}

/// Runs `body`, converting any panic into a status code.
///
/// Every entry point goes through this. A panic unwinding into Swift or Kotlin is
/// undefined behaviour, and an editor that corrupts its host process on a bug is worse
/// than one that reports an error.
fn guard<F: FnOnce() -> TopoviumStatus>(body: F) -> TopoviumStatus {
    catch_unwind(AssertUnwindSafe(body)).unwrap_or(TopoviumStatus::InternalError)
}

/// Writes the Topovium version as a NUL-terminated UTF-8 string.
///
/// The caller owns the returned pointer and must release it with
/// [`topovium_string_free`].
///
/// Returns null if the version string cannot be allocated.
#[unsafe(no_mangle)]
pub extern "C" fn topovium_version() -> *mut c_char {
    catch_unwind(|| {
        CString::new(env!("CARGO_PKG_VERSION")).map_or(std::ptr::null_mut(), CString::into_raw)
    })
    .unwrap_or(std::ptr::null_mut())
}

/// Releases a string returned by this library.
///
/// # Safety
/// `pointer` must be null, or a pointer previously returned by a `topovium_*` function
/// in this library and not yet freed. Passing anything else is undefined behaviour.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn topovium_string_free(pointer: *mut c_char) {
    if pointer.is_null() {
        return;
    }
    // SAFETY: the caller contract above guarantees `pointer` came from
    // `CString::into_raw` in this library and has not been freed. Reconstructing the
    // `CString` transfers ownership back to Rust, which then drops it.
    let owned = unsafe { CString::from_raw(pointer) };
    drop(owned);
}

/// Reports the device's thermal state so the frame budget can react before the system
/// throttles.
///
/// # Safety
/// Safe to call from any thread. Takes no pointers.
#[unsafe(no_mangle)]
pub extern "C" fn topovium_set_thermal_state(state: TopoviumThermalState) -> TopoviumStatus {
    guard(|| {
        let _mapped: ThermalState = state.into();
        // Wiring into the live editor session lands with the session handle in 0.0.2.
        TopoviumStatus::Ok
    })
}

/// The render resolution scale a given thermal state implies, as a convenience for
/// shells that size their drawable themselves.
#[unsafe(no_mangle)]
pub extern "C" fn topovium_resolution_scale_for_thermal_state(state: TopoviumThermalState) -> f32 {
    catch_unwind(|| ThermalState::from(state).resolution_scale()).unwrap_or(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn version_round_trips_and_frees_without_leaking() {
        let pointer = topovium_version();
        assert!(!pointer.is_null());
        // SAFETY: `pointer` was just returned by `topovium_version` and is a valid,
        // NUL-terminated string that has not been freed.
        let text = unsafe { CStr::from_ptr(pointer) }
            .to_str()
            .unwrap()
            .to_owned();
        assert_eq!(text, env!("CARGO_PKG_VERSION"));
        // SAFETY: `pointer` came from `topovium_version` and has not been freed.
        unsafe { topovium_string_free(pointer) };
    }

    #[test]
    fn freeing_null_is_a_no_op_rather_than_a_crash() {
        // SAFETY: null is explicitly permitted by the function's contract.
        unsafe { topovium_string_free(std::ptr::null_mut()) };
    }

    #[test]
    fn thermal_states_map_to_the_same_scales_as_the_rust_api() {
        assert!(
            (topovium_resolution_scale_for_thermal_state(TopoviumThermalState::Serious)
                - ThermalState::Serious.resolution_scale())
            .abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn reporting_thermal_state_succeeds() {
        assert_eq!(
            topovium_set_thermal_state(TopoviumThermalState::Critical),
            TopoviumStatus::Ok
        );
    }
}
