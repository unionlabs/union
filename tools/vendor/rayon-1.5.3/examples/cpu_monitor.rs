use docopt::Docopt;
use std::io;
use std::process;

const USAGE: &str = "
Usage: cpu_monitor [options] <scenario>
       cpu_monitor --help

A test for monitoring how much CPU usage Rayon consumes under various
scenarios. This test is intended to be executed interactively, like so:

    cargo run --example cpu_monitor -- tasks_ended

The list of scenarios you can try are as follows:

- tasks_ended: after all tasks have finished, go to sleep
- task_stall_root: a root task stalls for a very long time
- task_stall_scope: a task in a scope stalls for a very long time

Options:
    -h, --help                   Show this message.
    -d N, --depth N              Control how hard the dummy task works [default: 27]
";

#[derive(serde::Deserialize)]
pub struct Args {
    arg_scenario: String,
    flag_depth: usize,
}

fn main() {
    let args: &Args = &Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    match &args.arg_scenario[..] {
        "tasks_ended" => tasks_ended(args),
        "task_stall_root" => task_stall_root(args),
        "task_stall_scope" => task_stall_scope(args),
        _ => {
            println!("unknown scenario: `{}`", args.arg_scenario);
            println!("try --help");
            process::exit(1);
        }
    }
}

fn wait_for_user() {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}

fn task(args: &Args) {
    fn join_recursively(n: usize) {
        if n == 0 {
            return;
        }
        rayon::join(|| join_recursively(n - 1), || join_recursively(n - 1));
    }

    println!("Starting heavy work at depth {}...wait.", args.flag_depth);
    join_recursively(args.flag_depth);
    println!("Heavy work done; check top. You should see CPU usage drop to zero soon.");
    println!("Press <enter> to quit...");
}

fn tasks_ended(args: &Args) {
    task(args);
    wait_for_user();
}

fn task_stall_root(args: &Args) {
    rayon::join(|| task(args), wait_for_user);
}

fn task_stall_scope(args: &Args) {
    rayon::scope(|scope| {
        scope.spawn(move |_| task(args));
        scope.spawn(move |_| wait_for_user());
    });
}
