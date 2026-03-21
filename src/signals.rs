use nix::sys::signal::{self, SaFlags, SigAction, SigHandler, SigSet, Signal};
use std::sync::atomic::{AtomicBool, Ordering};

pub static SIGWINCH_RECEIVED: AtomicBool = AtomicBool::new(false);
pub static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

pub fn setup_signal_handlers() -> Result<(), nix::Error> {
    let sigwinch_action = SigAction::new(
        SigHandler::Handler(handle_sigwinch),
        SaFlags::SA_RESTART,
        SigSet::empty(),
    );
    unsafe { signal::sigaction(Signal::SIGWINCH, &sigwinch_action)? };

    let shutdown_action = SigAction::new(
        SigHandler::Handler(handle_shutdown),
        SaFlags::SA_RESTART,
        SigSet::empty(),
    );
    unsafe { signal::sigaction(Signal::SIGHUP, &shutdown_action)? };
    unsafe { signal::sigaction(Signal::SIGTERM, &shutdown_action)? };

    Ok(())
}

extern "C" fn handle_sigwinch(_: libc::c_int) {
    SIGWINCH_RECEIVED.store(true, Ordering::SeqCst);
}

extern "C" fn handle_shutdown(_: libc::c_int) {
    SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
}

pub fn check_sigwinch() -> bool {
    SIGWINCH_RECEIVED.swap(false, Ordering::SeqCst)
}

pub fn check_shutdown() -> bool {
    SHUTDOWN_REQUESTED.load(Ordering::SeqCst)
}
