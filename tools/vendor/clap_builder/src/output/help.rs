#![cfg_attr(not(feature = "help"), allow(unused_variables))]

// Internal
use crate::builder::Command;
use crate::builder::StyledStr;
use crate::output::Usage;

/// Writes the parser help to the wrapped stream.
pub(crate) fn write_help(writer: &mut StyledStr, cmd: &Command, usage: &Usage<'_>, use_long: bool) {
    debug!("write_help");

    if let Some(h) = cmd.get_override_help() {
        writer.push_styled(h);
    } else {
        #[cfg(feature = "help")]
        {
            use super::AutoHelp;
            use super::HelpTemplate;
            if let Some(tmpl) = cmd.get_help_template() {
                HelpTemplate::new(writer, cmd, usage, use_long)
                    .write_templated_help(tmpl.as_styled_str());
            } else {
                AutoHelp::new(writer, cmd, usage, use_long).write_help();
            }
        }

        #[cfg(not(feature = "help"))]
        {
            debug!("write_help: no help, `Command::override_help` and `help` is missing");
        }
    }

    // Remove any extra lines caused by book keeping
    writer.trim();
    // Ensure there is still a trailing newline
    writer.push_str("\n");
}
