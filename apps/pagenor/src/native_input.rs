//! Shut down AppKit's text service while its client window is still alive.
//!
//! winit 0.30's view holds a weak NSWindow reference. AppKit may synchronously
//! send setMarkedText when deactivating a dead-key session during NSWindow dealloc.
//! Retain the native windows through run_native, and deactivate before deallocation.

use std::cell::RefCell;
use std::rc::Rc;

use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::{NSApplication, NSTextInputContext, NSWindow};

/// The main function keeps a clone until the native event loop has returned.
#[derive(Clone, Default)]
pub struct ShutdownWindows(Rc<RefCell<Vec<Retained<NSWindow>>>>);

impl ShutdownWindows {
    pub fn prepare(&self) {
        let Some(mtm) = MainThreadMarker::new() else {
            log::error!("Native text input shutdown requires the main thread");
            return;
        };
        let windows: Vec<_> = NSApplication::sharedApplication(mtm).windows().iter().collect();
        // Retain first: discard/deactivate can call the client synchronously.
        self.0.borrow_mut().clone_from(&windows);
        if let Some(input) = NSTextInputContext::currentInputContext(mtm) {
            input.discardMarkedText();
            input.deactivate();
        }
        for window in &windows {
            if let Some(input) = window.contentView().and_then(|view| view.inputContext()) {
                input.discardMarkedText();
                input.deactivate();
            }
            let _ = window.makeFirstResponder(None);
        }
        log::info!("Native text input stopped; retaining {} window(s) through event-loop shutdown", windows.len());
    }
}
