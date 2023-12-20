use super::frame_info::{GlobalFrameInfo, FRAME_INFO};
use backtrace::Backtrace;
use wasmer_types::{FrameInfo, TrapCode};
use wasmer_vm::Trap;

/// Given a `Trap`, this function returns the Wasm trace and the trap code.
pub fn get_trace_and_trapcode(trap: &Trap) -> (Vec<FrameInfo>, Option<TrapCode>) {
    let info = FRAME_INFO.read().unwrap();
    match &trap {
        // A user error
        Trap::User(_err) => (wasm_trace(&info, None, &Backtrace::new_unresolved()), None),
        // A trap caused by the VM being Out of Memory
        Trap::OOM { backtrace } => (wasm_trace(&info, None, backtrace), None),
        // A trap caused by an error on the generated machine code for a Wasm function
        Trap::Wasm {
            pc,
            signal_trap,
            backtrace,
        } => {
            let trap_code = info
                .lookup_trap_info(*pc)
                .map_or(signal_trap.unwrap_or(TrapCode::StackOverflow), |info| {
                    info.trap_code
                });

            (wasm_trace(&info, Some(*pc), backtrace), Some(trap_code))
        }
        // A trap triggered manually from the Wasmer runtime
        Trap::Lib {
            trap_code,
            backtrace,
        } => (wasm_trace(&info, None, backtrace), Some(*trap_code)),
    }
}

fn wasm_trace(
    info: &GlobalFrameInfo,
    trap_pc: Option<usize>,
    backtrace: &Backtrace,
) -> Vec<FrameInfo> {
    // Let's construct the trace
    backtrace
        .frames()
        .iter()
        .filter_map(|frame| {
            let pc = frame.ip() as usize;
            if pc == 0 {
                None
            } else {
                // Note that we need to be careful about the pc we pass in here to
                // lookup frame information. This program counter is used to
                // translate back to an original source location in the origin wasm
                // module. If this pc is the exact pc that the trap happened at,
                // then we look up that pc precisely. Otherwise backtrace
                // information typically points at the pc *after* the call
                // instruction (because otherwise it's likely a call instruction on
                // the stack). In that case we want to lookup information for the
                // previous instruction (the call instruction) so we subtract one as
                // the lookup.
                let pc_to_lookup = if Some(pc) == trap_pc { pc } else { pc - 1 };
                Some(pc_to_lookup)
            }
        })
        .filter_map(|pc| info.lookup_frame_info(pc))
        .collect::<Vec<_>>()
}
