use std::{
    io::{ErrorKind, Result},
    path::PathBuf,
    sync::OnceLock,
};

pub(crate) type SingleInstanceCallback = dyn FnMut(Vec<String>, String) + Send + Sync + 'static;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod platform_impl;
#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod platform_impl;
#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod platform_impl;

static ID: OnceLock<String> = OnceLock::new();

// pub fn init<F: FnMut(Vec<String>, String) + Send + Sync + 'static>(mut f: F) {
//     platform_impl::init(Box::new(move |args, cwd| {
//         #[cfg(feature = "deep-link")]
//         if let Some(deep_link) = app.try_state::<tauri_plugin_deep_link::DeepLink<R>>() {
//             deep_link.handle_cli_arguments(args.iter());
//         }
//         f(args, cwd)
//     }))
// }

/// This function is meant for use-cases where the default [`prepare()`] function can't be used.
///
/// # Errors
/// If ID was already set this functions returns an AlreadyExists error.
pub fn set_identifier(identifier: &str) -> Result<()> {
    ID.set(identifier.to_string())
        .map_err(|_| ErrorKind::AlreadyExists.into())
}

// Consider adding a function to register without starting the listener.

/// Registers a handler for the given scheme.
///
/// ## Platform-specific:
///
/// - **macOS**: On macOS schemes must be defined in an Info.plist file, therefore this function only calls [`listen()`] without registering the scheme. This function can only be called once on macOS.
pub fn register(scheme: &str) -> Result<()> {
    platform_impl::register(scheme)
}

/// Starts the event listener without registering any schemes.
///
/// ## Platform-specific:
///
/// - **macOS**: This function can only be called once on macOS.
pub fn listen<F: FnMut(String) + Send + Sync + 'static>(handler: F) -> Result<()> {
    platform_impl::listen(handler)
}

/// Unregister a previously registered scheme.
///
/// ## Platform-specific:
///
/// - **macOS**: This function has no effect on macOS.
pub fn unregister(scheme: &str) -> Result<()> {
    platform_impl::unregister(scheme)
}

/// Checks if current instance is the primary instance.
/// Also sends the URL event data to the primary instance and stops the process afterwards.
///
/// ## Platform-specific:
///
/// - **macOS**: Only registers the identifier (only relevant in debug mode). It does not interact with the primary instance and does not exit the app.
pub fn prepare(identifier: &str) {
    platform_impl::prepare(identifier)
}

/// TODO: Windows-only for now
#[cfg(target_os = "windows")]
pub fn destroy() {
    platform_impl::destroy()
}

/// Helper to get current exe path
pub(crate) fn current_exe() -> std::io::Result<PathBuf> {
    let path = std::env::current_exe()?;

    path.canonicalize()
}
