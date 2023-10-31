use std::borrow::Cow;

use super::compiler::{Opdata, FormatStringIterator};
use super::x64data::Flags;

pub fn format_opdata_list(name: &str, data: &[Opdata]) -> String {
    let mut forms = Vec::new();
    for data in data {
        forms.extend(format_opdata(name, data));
    }
    forms.join("\n")
}

pub fn format_opdata(name: &str, data: &Opdata) -> Vec<String> {
    let opsizes = if data.flags.contains(Flags::AUTO_SIZE) {&b"qwd"[..]}
             else if data.flags.contains(Flags::AUTO_NO32) {&b"qw"[..]}
             else if data.flags.contains(Flags::AUTO_REXW) {&b"qd"[..]}
             else if data.flags.contains(Flags::AUTO_VEXL) {&b"ho"[..]}
             else if name == "monitorx"                    {&b"qwd"[..]}
             else                                          {&b"!"[..]};

    let mut forms = Vec::new();
    for opsize in opsizes.iter().cloned() {
        let mut buf = String::new();
        buf.push_str(">>> ");
        buf.push_str(name);
        let mut first = true;
        for (ty, size) in FormatStringIterator::new(data.args) {
            if first {
                buf.push_str(" ");
                first = false;
            } else {
                buf.push_str(", ");
            }
            buf.push_str(&format_arg(ty, size, opsize))
        }
        if data.flags.contains(Flags::X86_ONLY) {
            for _ in buf.len() .. 45 {
                buf.push(' ');
            }
            buf.push_str(" (x86 only)");
        }
        if !data.features.is_empty() {
            for _ in buf.len() .. 45 {
                buf.push(' ');
            }
            buf.push_str(&format!(" ({})", data.features));
        }
        forms.push(buf);
    }
    forms
}

static REGS: [&str; 16] = ["a",  "d",  "c",   "b",   "bp",  "sp",  "si",  "di",
                                   "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15"];
static SEGREGS: [&str; 6] = ["es", "cs", "ss", "ds", "fs", "gs"];

fn format_arg(ty: u8, mut size: u8, opsize: u8) -> Cow<'static, str> {
    if size == b'*' {
        size = if opsize == b'q' && (ty == b'i' || ty == b'o') {
            b'd'
        } else {
            opsize
        };
    }

    fn format_size(size: u8) -> &'static str {
        match size {
            b'b' => "8",
            b'w' => "16",
            b'd' => "32",
            b'f' => "48",
            b'q' => "64",
            b'p' => "80",
            b'o' => "128",
            b'h' => "256",
            _ => ""
        }
    }

    match ty {
        b'i' => format!("imm{}",      format_size(size)).into(),
        b'o' => format!("rel{}off",   format_size(size)).into(),
        b'm' => format!("mem{}",      format_size(size)).into(),
        b'k' => format!("vm32addr{}", format_size(size)).into(),
        b'l' => format!("vm64addr{}", format_size(size)).into(),
        b'r' => format!("reg{}",      format_size(size)).into(),
        b'f' => "st".into(),
        b'x' => "mm".into(),
        b'y' => (if size == b'h' {"ymm"} else {"xmm"}).into(),
        b's' => "segreg".into(),
        b'c' => "creg".into(),
        b'd' => "dreg".into(),
        b'b' => "bndreg".into(),
        b'v' => format!("reg/mem{}", format_size(size)).into(),
        b'u' => format!("mm/mem{}", format_size(size)).into(),
        b'w' => format!("{}mm/mem{}", if size == b'h' {"y"} else {"x"}, format_size(size)).into(),
        b'A'..=b'P' => {
            let i = ty as usize - 'A' as usize;
            match size {
                b'b' => if i < 4 { format!("{}l", REGS[i]).into() }
                   else if i < 8 { REGS[i].into() }
                   else          { format!("{}b", REGS[i]).into() },
                b'w' => if i < 4 { format!("{}x", REGS[i]).into() }
                   else if i < 8 { REGS[i].into() }
                   else          { format!("{}w", REGS[i]).into() },
                b'd' => if i < 4 { format!("e{}x",REGS[i]).into() }
                   else if i < 8 { format!("e{}", REGS[i]).into() }
                   else          { format!("{}d", REGS[i]).into() },
                b'q' => if i < 4 { format!("r{}x",REGS[i]).into() }
                   else          { format!("r{}", REGS[i]).into() },
                _ => panic!("invalid formatting data")
            }
        },
        b'Q'..=b'V' => SEGREGS[ty as usize - 'Q' as usize].into(),
        b'W' => "cr8".into(),
        b'X' => "st0".into(),
        _ => panic!("invalid formatting data")
    }
}

#[cfg(feature = "dynasm_opmap")]
pub fn create_opmap() -> String {
    let mut s = String::new();

    let mut mnemnonics: Vec<_> = super::x64data::mnemnonics().cloned().collect();
    mnemnonics.sort();

    for mnemnonic in mnemnonics {
        // get the data for this mnemnonic
        let data = super::x64data::get_mnemnonic_data(mnemnonic).unwrap();
        // format the data for the opmap docs
        let mut formats = data.into_iter()
            .map(|x| format_opdata(mnemnonic, x))
            .flat_map(|x| x)
            .map(|x| x.replace(">>> ", ""))
            .collect::<Vec<_>>();
        formats.sort();

        // push mnemnonic name as title
        s.push_str("### ");
        s.push_str(mnemnonic);
        s.push_str("\n```insref\n");

        // push the formats
        s.push_str(&formats.join("\n"));
        s.push_str("\n```\n");
    }
    s
}
