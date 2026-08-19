// SPDX-License-Identifier: GPL-3.0-or-later
package org.topovium.shell

import android.os.Build
import android.os.PowerManager
import android.util.Log

/**
 * The Android shell.
 *
 * Deliberately thin. Everything above the platform boundary -- interface, tools, scene,
 * renderer -- lives in Rust and is shared with desktop, iPad, and the browser. Adding
 * product behaviour here would create a second implementation only Android users get.
 */
object TopoviumCore {
    private const val TAG = "Topovium"

    init {
        System.loadLibrary("topovium_ffi")
    }

    /** Mirrors `TopoviumStatus` in `crates/ffi`. */
    enum class Status(val code: Int) {
        OK(0),
        INTERNAL_ERROR(1),
        NULL_ARGUMENT(2),
        INVALID_ARGUMENT(3);

        companion object {
            fun from(code: Int): Status = entries.firstOrNull { it.code == code } ?: INTERNAL_ERROR
        }
    }

    /** Mirrors `TopoviumThermalState` in `crates/ffi`. */
    enum class ThermalState(val code: Int) {
        NOMINAL(0),
        FAIR(1),
        SERIOUS(2),
        CRITICAL(3);

        companion object {
            /**
             * Maps Android's thermal status onto the core's.
             *
             * `THERMAL_STATUS_LIGHT` maps to [FAIR] rather than [NOMINAL] on purpose: the
             * useful moment to shed background work is before the system throttles.
             * Reacting afterwards produces a frame-rate cliff the user feels.
             */
            fun fromPowerManager(status: Int): ThermalState = when (status) {
                PowerManager.THERMAL_STATUS_NONE -> NOMINAL
                PowerManager.THERMAL_STATUS_LIGHT -> FAIR
                PowerManager.THERMAL_STATUS_MODERATE -> SERIOUS
                else -> CRITICAL
            }
        }
    }

    external fun version(): String

    external fun setThermalState(state: Int): Int

    /** Reports the device's thermal state so the frame budget can react in time. */
    fun reportThermalState(powerManager: PowerManager) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) return

        val state = ThermalState.fromPowerManager(powerManager.currentThermalStatus)
        val status = Status.from(setThermalState(state.code))
        if (status != Status.OK) {
            Log.w(TAG, "reporting thermal state failed: $status")
        }
    }

    /**
     * Registers for thermal updates.
     *
     * Polling would either miss the transition or waste power. The listener is how a
     * twenty-minute modelling session stays at a stable frame rate instead of running
     * fast and then throttling.
     */
    fun observeThermalState(powerManager: PowerManager) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) return

        powerManager.addThermalStatusListener { status ->
            val state = ThermalState.fromPowerManager(status)
            setThermalState(state.code)
        }
    }
}
