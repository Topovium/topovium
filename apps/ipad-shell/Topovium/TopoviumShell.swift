// SPDX-License-Identifier: GPL-3.0-or-later
//
// The iPadOS shell.
//
// Deliberately thin. Everything above the platform boundary -- interface, tools, scene,
// renderer -- lives in Rust and is shared with desktop, Android, and the browser. Adding
// product behaviour here would create a second implementation that only iPad users get,
// which is precisely the divergence this architecture exists to prevent.

import Foundation
import MetalKit
import UIKit

/// Mirrors `TopoviumStatus` in `crates/ffi`.
enum TopoviumStatus: Int32 {
    case ok = 0
    case internalError = 1
    case nullArgument = 2
    case invalidArgument = 3
}

/// Mirrors `TopoviumThermalState` in `crates/ffi`.
enum TopoviumThermalState: Int32 {
    case nominal = 0
    case fair = 1
    case serious = 2
    case critical = 3

    /// Maps the system's thermal state onto the core's.
    ///
    /// `.fair` maps to `.fair` rather than `.nominal` because on iPad the useful moment
    /// to start shedding background work is before the system reports serious pressure.
    /// Reacting after the throttle produces a visible cliff.
    init(processInfo state: ProcessInfo.ThermalState) {
        switch state {
        case .nominal: self = .nominal
        case .fair: self = .fair
        case .serious: self = .serious
        case .critical: self = .critical
        @unknown default: self = .fair
        }
    }
}

// Declared by `crates/ffi`, linked as a static library.
@_silgen_name("topovium_version")
func topovium_version() -> UnsafeMutablePointer<CChar>?

@_silgen_name("topovium_string_free")
func topovium_string_free(_ pointer: UnsafeMutablePointer<CChar>?)

@_silgen_name("topovium_set_thermal_state")
func topovium_set_thermal_state(_ state: Int32) -> Int32

/// Hosts the Metal surface the Rust core renders into.
final class ViewportController: UIViewController {
    private var metalLayer: CAMetalLayer {
        // Safe by construction: `layerClass` below guarantees the type.
        guard let layer = view.layer as? CAMetalLayer else {
            fatalError("layerClass must return CAMetalLayer")
        }
        return layer
    }

    override class var layerClass: AnyClass { CAMetalLayer.self }

    override func viewDidLoad() {
        super.viewDidLoad()

        metalLayer.device = MTLCreateSystemDefaultDevice()
        metalLayer.pixelFormat = .bgra8Unorm_srgb
        metalLayer.framebufferOnly = true

        // The stylus drives the active tool, so it must not be delayed by gesture
        // recognisers deciding whether a touch was a scroll.
        view.isMultipleTouchEnabled = true

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(thermalStateChanged),
            name: ProcessInfo.thermalStateDidChangeNotification,
            object: nil
        )
        reportThermalState()
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()
        // Physical pixels: the core budgets frames against the real drawable size, and
        // a logical size would silently under-report cost on a Retina display.
        metalLayer.drawableSize = CGSize(
            width: view.bounds.width * view.contentScaleFactor,
            height: view.bounds.height * view.contentScaleFactor
        )
    }

    @objc private func thermalStateChanged() {
        reportThermalState()
    }

    private func reportThermalState() {
        let state = TopoviumThermalState(processInfo: ProcessInfo.processInfo.thermalState)
        let status = topovium_set_thermal_state(state.rawValue)
        if status != TopoviumStatus.ok.rawValue {
            NSLog("topovium: reporting thermal state failed with status \(status)")
        }
    }

    /// The core's version, for the about screen and bug reports.
    static func coreVersion() -> String {
        guard let pointer = topovium_version() else { return "unavailable" }
        defer { topovium_string_free(pointer) }
        return String(cString: pointer)
    }
}

extension ViewportController {
    /// Forwarded to the core as pointer samples carrying pressure, tilt, and hover.
    ///
    /// Every sample keeps its own timestamp rather than being stamped on arrival:
    /// input-to-pixel latency is the metric this project is judged on, and measuring it
    /// requires knowing when the sample was actually taken.
    override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent?) {
        super.touchesMoved(touches, with: event)
        // Wired to the core in the 0.0.1 milestone issue "iPad shell: pen and touch input".
    }
}
