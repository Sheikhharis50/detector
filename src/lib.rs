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
    last_activity: Arc<Mutex<Instant>>,  // Track the last activity time
    stop_signal: Arc<AtomicBool>,  // Indicates if the thread should stop
}

impl ActivityState {
    fn new() -> Self {
        ActivityState {
            active: Arc::new(AtomicBool::new(false)),
            last_activity: Arc::new(Mutex::new(Instant::now())),
            stop_signal: Arc::new(AtomicBool::new(false)),
        }
    }
}

// Global state holder for activity detection
lazy_static! {
    static ref STATE: Mutex<Option<ActivityState>> = Mutex::new(None);
}

// Function to start the event listener in a new thread with custom duration
fn start_listener(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let duration_secs = cx.argument::<JsNumber>(0)?.value(&mut cx) as u64;  // Get the duration argument in seconds

    let mut state_lock = STATE.lock().unwrap();
    if state_lock.is_some() {
        return cx.throw_error("Listener already running");
    }

    let state = ActivityState::new();
    let active_clone = Arc::clone(&state.active);
    let last_activity_clone = Arc::clone(&state.last_activity);
    let stop_signal_clone = Arc::clone(&state.stop_signal);

    // Spawn a thread to listen for user activity
    thread::spawn(move || {
        // The callback triggered by `rdev::listen`
        let callback = {
            let active_clone_inner = Arc::clone(&active_clone);  // Clone for the callback
            let last_activity_inner = Arc::clone(&last_activity_clone);  // Clone for the callback
            move |event: Event| {
                match event.event_type {
                    EventType::KeyPress(_) | EventType::MouseMove { .. } | EventType::ButtonPress(_) => {
                        active_clone_inner.store(true, Ordering::Relaxed);
                        let mut last_activity = last_activity_inner.lock().unwrap();
                        *last_activity = Instant::now();  // Update the last activity time
                    },
                    _ => {}
                }
            }
        };

        // Start listening for events
        if let Err(error) = listen(callback) {
            println!("Error: {:?}", error);
        }
    });

    // Spawn another thread to check inactivity based on custom duration
    let active_clone_for_inactivity = Arc::clone(&state.active);
    let last_activity_clone_for_inactivity = Arc::clone(&state.last_activity);
    thread::spawn(move || {
        while !stop_signal_clone.load(Ordering::Relaxed) {
            {
                let last_activity = last_activity_clone_for_inactivity.lock().unwrap();
                if last_activity.elapsed() > Duration::from_secs(duration_secs) {
                    active_clone_for_inactivity.store(false, Ordering::Relaxed);  // Set active to false after the custom duration
                }
            }
            thread::sleep(Duration::from_secs(1));  // Check every second
        }

        println!("Inactivity check stopped.");
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
    cx.export_function("start", start_listener)?;
    cx.export_function("isActive", is_user_active)?;
    cx.export_function("stop", stop_listener)?;
    Ok(())
}