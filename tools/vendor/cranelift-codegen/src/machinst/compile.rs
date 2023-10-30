//! Compilation backend pipeline: optimized IR to VCode / binemit.

use crate::ir::Function;
use crate::isa::TargetIsa;
use crate::log::DeferredDisplay;
use crate::machinst::*;
use crate::settings;
use crate::timing;

use regalloc::{allocate_registers_with_opts, Algorithm, Options, PrettyPrint};

/// Compile the given function down to VCode with allocated registers, ready
/// for binary emission.
pub fn compile<B: LowerBackend + TargetIsa>(
    f: &Function,
    b: &B,
    abi: Box<dyn ABICallee<I = B::MInst>>,
    reg_universe: &RealRegUniverse,
    emit_info: <B::MInst as MachInstEmit>::Info,
) -> CodegenResult<VCode<B::MInst>>
where
    B::MInst: PrettyPrint,
{
    // Compute lowered block order.
    let block_order = BlockLoweringOrder::new(f);
    // Build the lowering context.
    let lower = Lower::new(f, abi, emit_info, block_order)?;
    // Lower the IR.
    let (mut vcode, stack_map_request_info) = {
        let _tt = timing::vcode_lower();
        lower.lower(b)?
    };

    // Creating the vcode string representation may be costly for large functions, so defer its
    // rendering.
    log::trace!(
        "vcode from lowering: \n{}",
        DeferredDisplay::new(|| vcode.show_rru(Some(reg_universe)))
    );

    // Perform register allocation.
    let (run_checker, algorithm) = match vcode.flags().regalloc() {
        settings::Regalloc::Backtracking => (false, Algorithm::Backtracking(Default::default())),
        settings::Regalloc::BacktrackingChecked => {
            (true, Algorithm::Backtracking(Default::default()))
        }
        settings::Regalloc::ExperimentalLinearScan => {
            (false, Algorithm::LinearScan(Default::default()))
        }
        settings::Regalloc::ExperimentalLinearScanChecked => {
            (true, Algorithm::LinearScan(Default::default()))
        }
    };

    #[cfg(feature = "regalloc-snapshot")]
    {
        use std::fs;
        use std::path::Path;
        if let Some(path) = std::env::var("SERIALIZE_REGALLOC").ok() {
            let snapshot = regalloc::IRSnapshot::from_function(&vcode, reg_universe);
            let serialized = bincode::serialize(&snapshot).expect("couldn't serialize snapshot");

            let file_path = Path::new(&path).join(Path::new(&format!("ir{}.bin", f.name)));
            fs::write(file_path, &serialized).expect("couldn't write IR snapshot file");
        }
    }

    // If either there are no reference-typed values, or else there are
    // but there are no safepoints at which we need to know about them,
    // then we don't need stack maps.
    let sri = if stack_map_request_info.reftyped_vregs.len() > 0
        && stack_map_request_info.safepoint_insns.len() > 0
    {
        Some(&stack_map_request_info)
    } else {
        None
    };

    let result = {
        let _tt = timing::regalloc();
        allocate_registers_with_opts(
            &mut vcode,
            reg_universe,
            sri,
            Options {
                run_checker,
                algorithm,
            },
        )
        .map_err(|err| {
            log::error!(
                "Register allocation error for vcode\n{}\nError: {:?}",
                vcode.show_rru(Some(reg_universe)),
                err
            );
            err
        })
        .expect("register allocation")
    };

    // Reorder vcode into final order and copy out final instruction sequence
    // all at once. This also inserts prologues/epilogues.
    {
        let _tt = timing::vcode_post_ra();
        vcode.replace_insns_from_regalloc(result);
    }

    log::trace!(
        "vcode after regalloc: final version:\n{}",
        DeferredDisplay::new(|| vcode.show_rru(Some(reg_universe)))
    );

    Ok(vcode)
}
