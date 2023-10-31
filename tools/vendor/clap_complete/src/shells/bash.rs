use std::{fmt::Write as _, io::Write};

use clap::*;

use crate::generator::{utils, Generator};

/// Generate bash completion file
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Bash;

impl Generator for Bash {
    fn file_name(&self, name: &str) -> String {
        format!("{name}.bash")
    }

    fn generate(&self, cmd: &Command, buf: &mut dyn Write) {
        let bin_name = cmd
            .get_bin_name()
            .expect("crate::generate should have set the bin_name");

        w!(
            buf,
            format!(
                "_{name}() {{
    local i cur prev opts cmd
    COMPREPLY=()
    cur=\"${{COMP_WORDS[COMP_CWORD]}}\"
    prev=\"${{COMP_WORDS[COMP_CWORD-1]}}\"
    cmd=\"\"
    opts=\"\"

    for i in ${{COMP_WORDS[@]}}
    do
        case \"${{cmd}},${{i}}\" in
            \",$1\")
                cmd=\"{cmd}\"
                ;;{subcmds}
            *)
                ;;
        esac
    done

    case \"${{cmd}}\" in
        {cmd})
            opts=\"{name_opts}\"
            if [[ ${{cur}} == -* || ${{COMP_CWORD}} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W \"${{opts}}\" -- \"${{cur}}\") )
                return 0
            fi
            case \"${{prev}}\" in{name_opts_details}
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W \"${{opts}}\" -- \"${{cur}}\") )
            return 0
            ;;{subcmd_details}
    esac
}}

complete -F _{name} -o bashdefault -o default {name}
",
                name = bin_name,
                cmd = bin_name.replace('-', "__"),
                name_opts = all_options_for_path(cmd, bin_name),
                name_opts_details = option_details_for_path(cmd, bin_name),
                subcmds = all_subcommands(cmd),
                subcmd_details = subcommand_details(cmd)
            )
            .as_bytes()
        );
    }
}

fn all_subcommands(cmd: &Command) -> String {
    debug!("all_subcommands");

    fn add_command(
        parent_fn_name: &str,
        cmd: &Command,
        subcmds: &mut Vec<(String, String, String)>,
    ) {
        let fn_name = format!(
            "{parent_fn_name}__{cmd_name}",
            parent_fn_name = parent_fn_name,
            cmd_name = cmd.get_name().to_string().replace('-', "__")
        );
        subcmds.push((
            parent_fn_name.to_string(),
            cmd.get_name().to_string(),
            fn_name.clone(),
        ));
        for alias in cmd.get_visible_aliases() {
            subcmds.push((
                parent_fn_name.to_string(),
                alias.to_string(),
                fn_name.clone(),
            ));
        }
        for subcmd in cmd.get_subcommands() {
            add_command(&fn_name, subcmd, subcmds);
        }
    }
    let mut subcmds = vec![];
    let fn_name = cmd.get_name().replace('-', "__");
    for subcmd in cmd.get_subcommands() {
        add_command(&fn_name, subcmd, &mut subcmds);
    }
    subcmds.sort();

    let mut cases = vec![String::new()];
    for (parent_fn_name, name, fn_name) in subcmds {
        cases.push(format!(
            "{parent_fn_name},{name})
                cmd=\"{fn_name}\"
                ;;",
        ));
    }

    cases.join("\n            ")
}

fn subcommand_details(cmd: &Command) -> String {
    debug!("subcommand_details");

    let mut subcmd_dets = vec![String::new()];
    let mut scs = utils::all_subcommands(cmd)
        .iter()
        .map(|x| x.1.replace(' ', "__"))
        .collect::<Vec<_>>();

    scs.sort();

    subcmd_dets.extend(scs.iter().map(|sc| {
        format!(
            "{subcmd})
            opts=\"{sc_opts}\"
            if [[ ${{cur}} == -* || ${{COMP_CWORD}} -eq {level} ]] ; then
                COMPREPLY=( $(compgen -W \"${{opts}}\" -- \"${{cur}}\") )
                return 0
            fi
            case \"${{prev}}\" in{opts_details}
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W \"${{opts}}\" -- \"${{cur}}\") )
            return 0
            ;;",
            subcmd = sc.replace('-', "__"),
            sc_opts = all_options_for_path(cmd, sc),
            level = sc.split("__").map(|_| 1).sum::<u64>(),
            opts_details = option_details_for_path(cmd, sc)
        )
    }));

    subcmd_dets.join("\n        ")
}

fn option_details_for_path(cmd: &Command, path: &str) -> String {
    debug!("option_details_for_path: path={path}");

    let p = utils::find_subcommand_with_path(cmd, path.split("__").skip(1).collect());
    let mut opts = vec![String::new()];

    for o in p.get_opts() {
        if let Some(longs) = o.get_long_and_visible_aliases() {
            opts.extend(longs.iter().map(|long| {
                format!(
                    "--{})
                    COMPREPLY=({})
                    return 0
                    ;;",
                    long,
                    vals_for(o)
                )
            }));
        }

        if let Some(shorts) = o.get_short_and_visible_aliases() {
            opts.extend(shorts.iter().map(|short| {
                format!(
                    "-{})
                    COMPREPLY=({})
                    return 0
                    ;;",
                    short,
                    vals_for(o)
                )
            }));
        }
    }

    opts.join("\n                ")
}

fn vals_for(o: &Arg) -> String {
    debug!("vals_for: o={}", o.get_id());

    if let Some(vals) = crate::generator::utils::possible_values(o) {
        format!(
            "$(compgen -W \"{}\" -- \"${{cur}}\")",
            vals.iter()
                .filter(|pv| !pv.is_hide_set())
                .map(|n| n.get_name())
                .collect::<Vec<_>>()
                .join(" ")
        )
    } else if o.get_value_hint() == ValueHint::Other {
        String::from("\"${cur}\"")
    } else {
        String::from("$(compgen -f \"${cur}\")")
    }
}

fn all_options_for_path(cmd: &Command, path: &str) -> String {
    debug!("all_options_for_path: path={path}");

    let p = utils::find_subcommand_with_path(cmd, path.split("__").skip(1).collect());

    let mut opts = String::new();
    for short in utils::shorts_and_visible_aliases(p) {
        write!(&mut opts, "-{short} ").unwrap();
    }
    for long in utils::longs_and_visible_aliases(p) {
        write!(&mut opts, "--{long} ").unwrap();
    }
    for pos in p.get_positionals() {
        if let Some(vals) = utils::possible_values(pos) {
            for value in vals {
                write!(&mut opts, "{} ", value.get_name()).unwrap();
            }
        } else {
            write!(&mut opts, "{pos} ").unwrap();
        }
    }
    for (sc, _) in utils::subcommands(p) {
        write!(&mut opts, "{sc} ").unwrap();
    }
    opts.pop();

    opts
}
