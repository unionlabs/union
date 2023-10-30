use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let pointer_width = env::var("CARGO_CFG_TARGET_POINTER_WIDTH");
    let u64_digit = pointer_width.as_ref().map(String::as_str) == Ok("64");
    if u64_digit {
        autocfg::emit("u64_digit");
    }
    let ac = autocfg::new();
    let std = if ac.probe_sysroot_crate("std") {
        "std"
    } else {
        "core"
    };
    if ac.probe_path(&format!("{}::convert::TryFrom", std)) {
        autocfg::emit("has_try_from");
    }

    if let Ok(target_arch) = env::var("CARGO_CFG_TARGET_ARCH") {
        if target_arch == "x86_64" || target_arch == "x86" {
            let digit = if u64_digit { "u64" } else { "u32" };

            let addcarry = format!("{}::arch::{}::_addcarry_{}", std, target_arch, digit);
            if ac.probe_path(&addcarry) {
                autocfg::emit("use_addcarry");
            }
        }
    }

    autocfg::rerun_path("build.rs");

    write_radix_bases().unwrap();
}

/// Write tables of the greatest power of each radix for the given bit size.  These are returned
/// from `biguint::get_radix_base` to batch the multiplication/division of radix conversions on
/// full `BigUint` values, operating on primitive integers as much as possible.
///
/// e.g. BASES_16[3] = (59049, 10) // 3¹⁰ fits in u16, but 3¹¹ is too big
///      BASES_32[3] = (3486784401, 20)
///      BASES_64[3] = (12157665459056928801, 40)
///
/// Powers of two are not included, just zeroed, as they're implemented with shifts.
fn write_radix_bases() -> Result<(), Box<dyn Error>> {
    let out_dir = env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("radix_bases.rs");
    let mut f = File::create(&dest_path)?;

    for &bits in &[16, 32, 64] {
        let max = if bits < 64 {
            (1 << bits) - 1
        } else {
            std::u64::MAX
        };

        writeln!(f, "#[deny(overflowing_literals)]")?;
        writeln!(
            f,
            "pub(crate) static BASES_{bits}: [(u{bits}, usize); 257] = [",
            bits = bits
        )?;
        for radix in 0u64..257 {
            let (base, power) = if radix == 0 || radix.is_power_of_two() {
                (0, 0)
            } else {
                let mut power = 1;
                let mut base = radix;

                while let Some(b) = base.checked_mul(radix) {
                    if b > max {
                        break;
                    }
                    base = b;
                    power += 1;
                }
                (base, power)
            };
            writeln!(f, "    ({}, {}), // {}", base, power, radix)?;
        }
        writeln!(f, "];")?;
    }

    Ok(())
}
