use std::io::Write;

use clap::builder::StyledStr;
use clap::*;

use crate::generator::{utils, Generator};
use crate::INTERNAL_ERROR_MSG;

/// Generate powershell completion file
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct PowerShell;

impl Generator for PowerShell {
    fn file_name(&self, name: &str) -> String {
        format!("_{name}.ps1")
    }

    fn generate(&self, cmd: &Command, buf: &mut dyn Write) {
        let bin_name = cmd
            .get_bin_name()
            .expect("crate::generate should have set the bin_name");

        let subcommands_cases = generate_inner(cmd, "");

        let result = format!(
            r#"
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName '{bin_name}' -ScriptBlock {{
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        '{bin_name}'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {{
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {{
                break
        }}
        $element.Value
    }}) -join ';'

    $completions = @(switch ($command) {{{subcommands_cases}
    }})

    $completions.Where{{ $_.CompletionText -like "$wordToComplete*" }} |
        Sort-Object -Property ListItemText
}}
"#
        );

        w!(buf, result.as_bytes());
    }
}

// Escape string inside single quotes
fn escape_string(string: &str) -> String {
    string.replace('\'', "''")
}

fn get_tooltip<T: ToString>(help: Option<&StyledStr>, data: T) -> String {
    match help {
        Some(help) => escape_string(&help.to_string()),
        _ => data.to_string(),
    }
}

fn generate_inner(p: &Command, previous_command_name: &str) -> String {
    debug!("generate_inner");

    let command_name = if previous_command_name.is_empty() {
        p.get_bin_name().expect(INTERNAL_ERROR_MSG).to_string()
    } else {
        format!("{};{}", previous_command_name, &p.get_name())
    };

    let mut completions = String::new();
    let preamble = String::from("\n            [CompletionResult]::new(");

    for option in p.get_opts() {
        generate_aliases(&mut completions, &preamble, option);
    }

    for flag in utils::flags(p) {
        generate_aliases(&mut completions, &preamble, &flag);
    }

    for subcommand in p.get_subcommands() {
        let data = &subcommand.get_name();
        let tooltip = get_tooltip(subcommand.get_about(), data);

        completions.push_str(&preamble);
        completions.push_str(
            format!("'{data}', '{data}', [CompletionResultType]::ParameterValue, '{tooltip}')")
                .as_str(),
        );
    }

    let mut subcommands_cases = format!(
        r"
        '{}' {{{}
            break
        }}",
        &command_name, completions
    );

    for subcommand in p.get_subcommands() {
        let subcommand_subcommands_cases = generate_inner(subcommand, &command_name);
        subcommands_cases.push_str(&subcommand_subcommands_cases);
    }

    subcommands_cases
}

fn generate_aliases(completions: &mut String, preamble: &String, arg: &Arg) {
    use std::fmt::Write as _;

    if let Some(aliases) = arg.get_short_and_visible_aliases() {
        let tooltip = get_tooltip(arg.get_help(), aliases[0]);
        for alias in aliases {
            let _ = write!(
                completions,
                "{preamble}'-{alias}', '{alias}{}', [CompletionResultType]::ParameterName, '{tooltip}')",
                // make PowerShell realize there is a difference between `-s` and `-S`
                if alias.is_uppercase() { " " } else { "" },
            );
        }
    }
    if let Some(aliases) = arg.get_long_and_visible_aliases() {
        let tooltip = get_tooltip(arg.get_help(), aliases[0]);
        for alias in aliases {
            let _ = write!(
                completions,
                "{preamble}'--{alias}', '{alias}', [CompletionResultType]::ParameterName, '{tooltip}')"
            );
        }
    }
}
