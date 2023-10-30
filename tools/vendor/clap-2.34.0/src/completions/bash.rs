// Std
use std::io::Write;

// Internal
use crate::{
    app::parser::Parser,
    args::{AnyArg, OptBuilder},
    completions,
};

pub struct BashGen<'a, 'b>
where
    'a: 'b,
{
    p: &'b Parser<'a, 'b>,
}

impl<'a, 'b> BashGen<'a, 'b> {
    pub fn new(p: &'b Parser<'a, 'b>) -> Self {
        BashGen { p }
    }

    pub fn generate_to<W: Write>(&self, buf: &mut W) {
        w!(
            buf,
            format!(
                r#"_{name}() {{
    local i cur prev opts cmds
    COMPREPLY=()
    cur="${{COMP_WORDS[COMP_CWORD]}}"
    prev="${{COMP_WORDS[COMP_CWORD-1]}}"
    cmd=""
    opts=""

    for i in ${{COMP_WORDS[@]}}
    do
        case "${{i}}" in
            {name})
                cmd="{name}"
                ;;
            {subcmds}
            *)
                ;;
        esac
    done

    case "${{cmd}}" in
        {name})
            opts="{name_opts}"
            if [[ ${{cur}} == -* || ${{COMP_CWORD}} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${{opts}}" -- "${{cur}}") )
                return 0
            fi
            case "${{prev}}" in
                {name_opts_details}
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${{opts}}" -- "${{cur}}") )
            return 0
            ;;
        {subcmd_details}
    esac
}}

complete -F _{name} -o bashdefault -o default {name}
"#,
                name = self.p.meta.bin_name.as_ref().unwrap(),
                name_opts = self.all_options_for_path(self.p.meta.bin_name.as_ref().unwrap()),
                name_opts_details =
                    self.option_details_for_path(self.p.meta.bin_name.as_ref().unwrap()),
                subcmds = self.all_subcommands(),
                subcmd_details = self.subcommand_details()
            )
            .as_bytes()
        );
    }

    fn all_subcommands(&self) -> String {
        debugln!("BashGen::all_subcommands;");
        let mut subcmds = String::new();
        let scs = completions::all_subcommand_names(self.p);

        for sc in &scs {
            subcmds = format!(
                r#"{}
            {name})
                cmd+="__{fn_name}"
                ;;"#,
                subcmds,
                name = sc,
                fn_name = sc.replace("-", "__")
            );
        }

        subcmds
    }

    fn subcommand_details(&self) -> String {
        debugln!("BashGen::subcommand_details;");
        let mut subcmd_dets = String::new();
        let mut scs = completions::get_all_subcommand_paths(self.p, true);
        scs.sort();
        scs.dedup();

        for sc in &scs {
            subcmd_dets = format!(
                r#"{}
        {subcmd})
            opts="{sc_opts}"
            if [[ ${{cur}} == -* || ${{COMP_CWORD}} -eq {level} ]] ; then
                COMPREPLY=( $(compgen -W "${{opts}}" -- "${{cur}}") )
                return 0
            fi
            case "${{prev}}" in
                {opts_details}
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${{opts}}" -- "${{cur}}") )
            return 0
            ;;"#,
                subcmd_dets,
                subcmd = sc.replace("-", "__"),
                sc_opts = self.all_options_for_path(&*sc),
                level = sc.split("__").count(),
                opts_details = self.option_details_for_path(&*sc)
            );
        }

        subcmd_dets
    }

    fn option_details_for_path(&self, path: &str) -> String {
        debugln!("BashGen::option_details_for_path: path={}", path);
        let mut p = self.p;
        for sc in path.split("__").skip(1) {
            debugln!("BashGen::option_details_for_path:iter: sc={}", sc);
            p = &find_subcmd!(p, sc).unwrap().p;
        }
        let mut opts = String::new();
        for o in p.opts() {
            if let Some(l) = o.s.long {
                opts = format!(
                    "{}
                --{})
                    COMPREPLY=({})
                    return 0
                    ;;",
                    opts,
                    l,
                    self.vals_for(o)
                );
            }
            if let Some(s) = o.s.short {
                opts = format!(
                    "{}
                    -{})
                    COMPREPLY=({})
                    return 0
                    ;;",
                    opts,
                    s,
                    self.vals_for(o)
                );
            }
        }
        opts
    }

    fn vals_for(&self, o: &OptBuilder) -> String {
        debugln!("BashGen::vals_for: o={}", o.b.name);
        if let Some(vals) = o.possible_vals() {
            format!(r#"$(compgen -W "{}" -- "${{cur}}")"#, vals.join(" "))
        } else {
            String::from(r#"$(compgen -f "${cur}")"#)
        }
    }

    fn all_options_for_path(&self, path: &str) -> String {
        debugln!("BashGen::all_options_for_path: path={}", path);
        let mut p = self.p;
        for sc in path.split("__").skip(1) {
            debugln!("BashGen::all_options_for_path:iter: sc={}", sc);
            p = &find_subcmd!(p, sc).unwrap().p;
        }
        let mut opts = shorts!(p).fold(String::new(), |acc, s| format!("{} -{}", acc, s));
        opts = format!(
            "{} {}",
            opts,
            longs!(p).fold(String::new(), |acc, l| format!("{} --{}", acc, l))
        );
        opts = format!(
            "{} {}",
            opts,
            p.positionals
                .values()
                .fold(String::new(), |acc, p| format!("{} {}", acc, p))
        );
        opts = format!(
            "{} {}",
            opts,
            p.subcommands
                .iter()
                .fold(String::new(), |acc, s| format!("{} {}", acc, s.p.meta.name))
        );
        for sc in &p.subcommands {
            if let Some(ref aliases) = sc.p.meta.aliases {
                opts = format!(
                    "{} {}",
                    opts,
                    aliases
                        .iter()
                        .map(|&(n, _)| n)
                        .fold(String::new(), |acc, a| format!("{} {}", acc, a))
                );
            }
        }
        opts
    }
}
