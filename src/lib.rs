#[macro_use]
extern crate lazy_static;

use neon::prelude::*;
use rdev::{Event, EventType, listen};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::{Duration, Instant};

// Structure to hold shared state
struct ActivityState {
    active: Arc<AtomicBool>,  // Indicates if the user is active
    stop_signal: Arc<AtomicBool>,  // Indicates if the thread should stop
}

impl ActivityState {
    fn new() -> Self {
        ActivityState {
            active: Arc::new(AtomicBool::new(false)),
            stop_signal: Arc::new(AtomicBool::new(false)),
        }
    }
}

// Global state holder for activity detection
lazy_static! {
    static ref STATE: Mutex<Option<ActivityState>> = Mutex::new(None);
}

// Function to start the event listener in a new thread
fn start_listener(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // Create or retrieve the current activity state
    let mut state_lock = STATE.lock().unwrap();
    if state_lock.is_some() {
        return cx.throw_error("Listener already running");
    }

    let state = ActivityState::new();
    let active_clone = Arc::clone(&state.active);
    let stop_signal_clone = Arc::clone(&state.stop_signal);

    // Spawn a thread to listen for user activity
    thread::spawn(move || {
        let mut last_activity = Instant::now();

        // The callback triggered by `rdev::listen`
        let callback = {
            let active_clone_inner = Arc::clone(&active_clone); // Clone here
            move |event: Event| {
                match event.event_type {
                    EventType::KeyPress(_) | EventType::MouseMove { .. } | EventType::ButtonPress(_) => {
                        active_clone_inner.store(true, Ordering::Relaxed);
                        last_activity = Instant::now();  // Update the last activity time
                    },
                    _ => {}
                }
            }
        };

        // Listen for events while the stop signal is not set
        if let Err(error) = listen(callback) {
            println!("Error: {:?}", error);
        }

        // Periodically check if the user has been inactive
        while !stop_signal_clone.load(Ordering::Relaxed) {
            if last_activity.elapsed() > Duration::from_secs(10) {
                active_clone.store(false, Ordering::Relaxed);  // Set active to false if 10 seconds of inactivity
            }
            thread::sleep(Duration::from_secs(1));  // Check every second
        }

        println!("Listener thread stopped.");
    });

    *state_lock = Some(state);
    Ok(cx.undefined())
}

// Function to check if the user is currently active
fn is_user_active(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let state_lock = STATE.lock().unwrap();
    if let Some(state) = &*state_lock {
        let is_active = state.active.load(Ordering::Relaxed);
        Ok(cx.boolean(is_active))
    } else {
        cx.throw_error("Listener is not running")
    }
}

// Function to stop the listener thread
fn stop_listener(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut state_lock = STATE.lock().unwrap();
    if let Some(state) = &mut *state_lock {
        state.stop_signal.store(true, Ordering::Relaxed);
        *state_lock = None;  // Reset the state
    }
    Ok(cx.undefined())
}

// Register the functions for the Neon module
#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("startListener", start_listener)?;
    cx.export_function("isUserActive", is_user_active)?;
    cx.export_function("stopListener", stop_listener)?;
    Ok(())
}