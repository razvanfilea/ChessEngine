use chess_engine::uci::UciState;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub type UciOutputCallback = unsafe extern "C" fn(*const u8, usize);

struct OutputState {
    cb: Option<UciOutputCallback>,
    queue: VecDeque<String>,
}

pub struct FfiState {
    uci: UciState,
    output: Arc<Mutex<OutputState>>,
}

const CMD_BUFFER_SIZE: usize = 4096;
static mut CMD_BUFFER: [u8; CMD_BUFFER_SIZE] = [0; CMD_BUFFER_SIZE];

/// Creates a new `FfiState` instance allocated on the heap.
#[unsafe(no_mangle)]
pub extern "C" fn uci_new() -> *mut FfiState {
    let output = Arc::new(Mutex::new(OutputState {
        cb: None,
        queue: VecDeque::new(),
    }));

    let out_clone = output.clone();
    let uci = UciState::new(move |line: String| {
        if let Ok(mut state) = out_clone.lock() {
            if let Some(cb) = state.cb {
                unsafe { cb(line.as_ptr(), line.len()) };
            } else {
                state.queue.push_back(line);
            }
        }
    });

    Box::into_raw(Box::new(FfiState { uci, output }))
}

/// Frees an `FfiState` instance previously created by `uci_new`.
///
/// # Safety
/// `ptr` must be a valid pointer returned by `uci_new` that has not been freed yet, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uci_free(ptr: *mut FfiState) {
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(ptr));
        }
    }
}

/// Sets an optional output callback for UCI responses (e.g. `uciok`, `info ...`, `bestmove ...`).
///
/// # Safety
/// `ptr` must point to a valid, live `FfiState` or be null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uci_set_output_callback(
    ptr: *mut FfiState,
    cb: Option<UciOutputCallback>,
) {
    if let Some(state) = unsafe { ptr.as_mut() }
        && let Ok(mut out) = state.output.lock()
    {
        out.cb = cb;
    }
}

/// Returns a pointer to the static command buffer.
/// Embedders can write UTF-8 command bytes directly into this buffer and call `uci_send_cmd`,
/// avoiding any memory allocations.
#[unsafe(no_mangle)]
pub extern "C" fn uci_get_cmd_buffer() -> *mut u8 {
    std::ptr::addr_of_mut!(CMD_BUFFER).cast::<u8>()
}

/// Executes a command written into the static command buffer up to `len` bytes.
///
/// # Safety
/// `ptr` must point to a valid, live `FfiState`.
/// The command buffer must contain at least `len` initialized bytes of valid UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uci_send_cmd(ptr: *mut FfiState, len: usize) -> bool {
    if ptr.is_null() || len > CMD_BUFFER_SIZE {
        return false;
    }
    let state = unsafe { &mut *ptr };
    let slice =
        unsafe { std::slice::from_raw_parts(std::ptr::addr_of!(CMD_BUFFER).cast::<u8>(), len) };
    if let Ok(s) = std::str::from_utf8(slice) {
        state.uci.process_command(s)
    } else {
        false
    }
}

/// Executes a UCI command from an arbitrary pointer and byte length.
///
/// # Safety
/// `ptr` must point to a valid, live `FfiState`.
/// `cmd_ptr` must point to at least `cmd_len` valid, initialized bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uci_command(
    ptr: *mut FfiState,
    cmd_ptr: *const u8,
    cmd_len: usize,
) -> bool {
    if ptr.is_null() || cmd_ptr.is_null() {
        return false;
    }
    let state = unsafe { &mut *ptr };
    let slice = unsafe { std::slice::from_raw_parts(cmd_ptr, cmd_len) };
    if let Ok(s) = std::str::from_utf8(slice) {
        state.uci.process_command(s)
    } else {
        false
    }
}

/// Reads the next queued output line into `out_ptr` up to `max_len` bytes.
/// Returns the number of bytes written, or 0 if no output is pending.
///
/// # Safety
/// `ptr` must point to a valid, live `FfiState`.
/// `out_ptr` must point to a buffer capable of holding at least `max_len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uci_read_output(
    ptr: *mut FfiState,
    out_ptr: *mut u8,
    max_len: usize,
) -> usize {
    if ptr.is_null() || out_ptr.is_null() || max_len == 0 {
        return 0;
    }
    let state = unsafe { &mut *ptr };
    if let Ok(mut out) = state.output.lock()
        && let Some(line) = out.queue.pop_front()
    {
        let bytes = line.as_bytes();
        let copy_len = bytes.len().min(max_len);
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), out_ptr, copy_len);
        }
        return copy_len;
    }
    0
}

/// Requests an active search to stop.
///
/// # Safety
/// `ptr` must point to a valid, live `FfiState` or be null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uci_stop(ptr: *mut FfiState) {
    if let Some(state) = unsafe { ptr.as_mut() } {
        state.uci.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uci_c_ffi_basic() {
        unsafe {
            let uci_ptr = uci_new();
            assert!(!uci_ptr.is_null());

            let cmd = b"isready";
            assert!(uci_command(uci_ptr, cmd.as_ptr(), cmd.len()));

            let mut out = [0u8; 128];
            let n = uci_read_output(uci_ptr, out.as_mut_ptr(), out.len());
            assert_eq!(&out[..n], b"readyok");

            uci_free(uci_ptr);
        }
    }

    #[test]
    fn test_uci_static_cmd_buffer() {
        unsafe {
            let uci_ptr = uci_new();
            assert!(!uci_ptr.is_null());

            let buf = uci_get_cmd_buffer();
            let cmd = b"uci";
            std::ptr::copy_nonoverlapping(cmd.as_ptr(), buf, cmd.len());
            assert!(uci_send_cmd(uci_ptr, cmd.len()));

            let mut out = [0u8; 512];
            let n = uci_read_output(uci_ptr, out.as_mut_ptr(), out.len());
            assert!(n > 0);
            let s = std::str::from_utf8(&out[..n]).unwrap();
            assert!(s.contains("id name lucky_chess"));

            uci_free(uci_ptr);
        }
    }
}
