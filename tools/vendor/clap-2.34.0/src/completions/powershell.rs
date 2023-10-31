// Std
use std::io::Write;

// Internal
use crate::{app::parser::Parser, INTERNAL_ERROR_MSG};

pub struct PowerShellGen<'a, 'b>
where
    'a: 'b,
{
    p: &'b Parser<'a, 'b>,
}

impl<'a, 'b> PowerShellGen<'a, 'b> {
    pub fn new(p: &'b Parser<'a, 'b>) -> Self {
        PowerShellGen { p }
    }

    pub fn generate_to<W: Write>(&self, buf: &mut W) {
        let bin_name = self.p.meta.bin_name.as_ref().unwrap();

        let mut names = vec![];
        let subcommands_cases = generate_inner(self.p, "", &mut names);

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
                $element.Value.StartsWith('-')) {{
                break
        }}
        $element.Value
    }}) -join ';'

    $completions = @(switch ($command) {{{subcommands_cases}
    }})

    $completions.Where{{ $_.CompletionText -like "$wordToComplete*" }} |
        Sort-Object -Property ListItemText
}}
"#,
            bin_name = bin_name,
            subcommands_cases = subcommands_cases
        );

        w!(buf, result.as_bytes());
    }
}

// Escape string inside single quotes
fn escape_string(string: &str) -> String {
    string.replace("'", "''")
}

fn get_tooltip<T: ToString>(help: Option<&str>, data: T) -> String {
    match help {
        Some(help) => escape_string(help),
        _ => data.to_string(),
    }
}

fn generate_inner<'a, 'b, 'p>(
    p: &'p Parser<'a, 'b>,
    previous_command_name: &str,
    names: &mut Vec<&'p str>,
) -> String {
    debugln!("PowerShellGen::generate_inner;");
    let command_name = if previous_command_name.is_empty() {
        p.meta.bin_name.as_ref().expect(INTERNAL_ERROR_MSG).clone()
    } else {
        format!("{};{}", previous_command_name, &p.meta.name)
    };

    let mut completions = String::new();
    let preamble = String::from("\n            [CompletionResult]::new(");

    for option in p.opts() {
        if let Some(data) = option.s.short {
            let tooltip = get_tooltip(option.b.help, data);
            completions.push_str(&preamble);
            completions.push_str(
                format!(
                    "'-{}', '{}', {}, '{}')",
                    data, data, "[CompletionResultType]::ParameterName", tooltip
                )
                .as_str(),
            );
        }
        if let Some(data) = option.s.long {
            let tooltip = get_tooltip(option.b.help, data);
            completions.push_str(&preamble);
            completions.push_str(
                format!(
                    "'--{}', '{}', {}, '{}')",
                    data, data, "[CompletionResultType]::ParameterName", tooltip
                )
                .as_str(),
            );
        }
    }

    for flag in p.flags() {
        if let Some(data) = flag.s.short {
            let tooltip = get_tooltip(flag.b.help, data);
            completions.push_str(&preamble);
            completions.push_str(
                format!(
                    "'-{}', '{}', {}, '{}')",
                    data, data, "[CompletionResultType]::ParameterName", tooltip
                )
                .as_str(),
            );
        }
        if let Some(data) = flag.s.long {
            let tooltip = get_tooltip(flag.b.help, data);
            completions.push_str(&preamble);
            completions.push_str(
                format!(
                    "'--{}', '{}', {}, '{}')",
                    data, data, "[CompletionResultType]::ParameterName", tooltip
                )
                .as_str(),
            );
        }
    }

    for subcommand in &p.subcommands {
        let data = &subcommand.p.meta.name;
        let tooltip = get_tooltip(subcommand.p.meta.about, data);
        completions.push_str(&preamble);
        completions.push_str(
            format!(
                "'{}', '{}', {}, '{}')",
                data, data, "[CompletionResultType]::ParameterValue", tooltip
            )
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

    for subcommand in &p.subcommands {
        let subcommand_subcommands_cases = generate_inner(&subcommand.p, &command_name, names);
        subcommands_cases.push_str(&subcommand_subcommands_cases);
    }

    subcommands_cases
}
