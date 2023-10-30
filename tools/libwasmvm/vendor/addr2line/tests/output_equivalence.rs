extern crate backtrace;
extern crate findshlibs;
extern crate rustc_test as test;

use std::env;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

use backtrace::Backtrace;
use findshlibs::{IterationControl, SharedLibrary, TargetSharedLibrary};
use test::{ShouldPanic, TestDesc, TestDescAndFn, TestFn, TestName};

#[inline(never)]
fn make_trace() -> Vec<String> {
    fn foo() -> Backtrace {
        bar()
    }
    #[inline(never)]
    fn bar() -> Backtrace {
        baz()
    }
    #[inline(always)]
    fn baz() -> Backtrace {
        Backtrace::new_unresolved()
    }

    let mut base_addr = None;
    TargetSharedLibrary::each(|lib| {
        base_addr = Some(lib.virtual_memory_bias().0 as isize);
        IterationControl::Break
    });
    let addrfix = -base_addr.unwrap();

    let trace = foo();
    trace
        .frames()
        .iter()
        .take(5)
        .map(|x| format!("{:p}", (x.ip() as *const u8).wrapping_offset(addrfix)))
        .collect()
}

fn run_cmd<P: AsRef<OsStr>>(exe: P, me: &Path, flags: Option<&str>, trace: &str) -> String {
    let mut cmd = Command::new(exe);
    cmd.env("LC_ALL", "C"); // GNU addr2line is localized, we aren't
    cmd.env("RUST_BACKTRACE", "1"); // if a child crashes, we want to know why

    if let Some(flags) = flags {
        cmd.arg(flags);
    }
    cmd.arg("--exe").arg(me).arg(trace);

    let output = cmd.output().unwrap();

    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap()
}

fn run_test(flags: Option<&str>) {
    let me = env::current_exe().unwrap();
    let mut exe = me.clone();
    assert!(exe.pop());
    if exe.file_name().unwrap().to_str().unwrap() == "deps" {
        assert!(exe.pop());
    }
    exe.push("examples");
    exe.push("addr2line");

    assert!(exe.is_file());

    let trace = make_trace();

    // HACK: GNU addr2line has a bug where looking up multiple addresses can cause the second
    // lookup to fail. Workaround by doing one address at a time.
    for addr in &trace {
        let theirs = run_cmd("addr2line", &me, flags, addr);
        let ours = run_cmd(&exe, &me, flags, addr);

        // HACK: GNU addr2line does not tidy up paths properly, causing double slashes to be printed.
        // We consider our behavior to be correct, so we fix their output to match ours.
        let theirs = theirs.replace("//", "/");

        assert!(
            theirs == ours,
            "Output not equivalent:

$ addr2line {0} --exe {1} {2}
{4}
$ {3} {0} --exe {1} {2}
{5}


",
            flags.unwrap_or(""),
            me.display(),
            trace.join(" "),
            exe.display(),
            theirs,
            ours
        );
    }
}

static FLAGS: &'static str = "aipsf";

fn make_tests() -> Vec<TestDescAndFn> {
    (0..(1 << FLAGS.len()))
        .map(|bits| {
            if bits == 0 {
                None
            } else {
                let mut param = String::new();
                param.push('-');
                for (i, flag) in FLAGS.chars().enumerate() {
                    if (bits & (1 << i)) != 0 {
                        param.push(flag);
                    }
                }
                Some(param)
            }
        })
        .map(|param| TestDescAndFn {
            desc: TestDesc {
                name: TestName::DynTestName(format!(
                    "addr2line {}",
                    param.as_ref().map_or("", String::as_str)
                )),
                ignore: false,
                should_panic: ShouldPanic::No,
                allow_fail: false,
            },
            testfn: TestFn::DynTestFn(Box::new(move || {
                run_test(param.as_ref().map(String::as_str))
            })),
        })
        .collect()
}

fn main() {
    if !cfg!(target_os = "linux") {
        return;
    }
    let args: Vec<_> = env::args().collect();
    test::test_main(&args, make_tests());
}
