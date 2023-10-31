#![allow(deprecated)]

// Internal
use crate::builder::Arg;
use crate::builder::ArgSettings;
use crate::INTERNAL_ERROR_MSG;

#[derive(PartialEq, Debug)]
enum UsageToken {
    Name,
    ValName,
    Short,
    Long,
    Help,
    Multiple,
    Unknown,
    Default,
}

#[derive(Debug)]
pub(crate) struct UsageParser<'help> {
    usage: &'help str,
    pos: usize,
    start: usize,
    prev: UsageToken,
    explicit_name_set: bool,
}

impl<'help> UsageParser<'help> {
    fn new(usage: &'help str) -> Self {
        debug!("new: usage={:?}", usage);
        UsageParser {
            usage,
            pos: 0,
            start: 0,
            prev: UsageToken::Unknown,
            explicit_name_set: false,
        }
    }

    pub(crate) fn from_usage(usage: &'help str) -> Self {
        debug!("UsageParser::from_usage");
        UsageParser::new(usage)
    }

    pub(crate) fn parse(mut self) -> Arg<'help> {
        debug!("UsageParser::parse");
        let mut arg = Arg::default();
        loop {
            debug!("UsageParser::parse:iter: pos={}", self.pos);
            self.stop_at(token);
            if let Some(&c) = self.usage.as_bytes().get(self.pos) {
                match c {
                    b'-' => self.short_or_long(&mut arg),
                    b'.' => self.multiple(&mut arg),
                    b'@' => self.default(&mut arg),
                    b'\'' => self.help(&mut arg),
                    _ => self.name(&mut arg),
                }
            } else {
                break;
            }
        }

        debug!("UsageParser::parse: vals...{:?}", arg.val_names);
        arg
    }

    fn name(&mut self, arg: &mut Arg<'help>) {
        debug!("UsageParser::name");
        if *self
            .usage
            .as_bytes()
            .get(self.pos)
            .expect(INTERNAL_ERROR_MSG)
            == b'<'
            && !self.explicit_name_set
        {
            arg.settings.set(ArgSettings::Required);
        }
        self.pos += 1;
        self.stop_at(name_end);
        let name = &self.usage[self.start..self.pos];
        if self.prev == UsageToken::Unknown {
            debug!("UsageParser::name: setting name...{}", name);
            arg.id = name.into();
            arg.name = name;
            if arg.long.is_none() && arg.short.is_none() {
                debug!("name: explicit name set...");
                self.explicit_name_set = true;
                self.prev = UsageToken::Name;
            }
        } else {
            debug!("UsageParser::name: setting val name...{}", name);
            if arg.val_names.is_empty() {
                arg.settings.set(ArgSettings::TakesValue);
            }
            let len = arg.val_names.len();
            arg.val_names.insert(len, name);
            self.prev = UsageToken::ValName;
        }
    }

    fn stop_at<F>(&mut self, f: F)
    where
        F: Fn(u8) -> bool,
    {
        debug!("UsageParser::stop_at");
        self.start = self.pos;
        self.pos += self.usage[self.start..]
            .bytes()
            .take_while(|&b| f(b))
            .count();
    }

    fn short_or_long(&mut self, arg: &mut Arg<'help>) {
        debug!("UsageParser::short_or_long");
        self.pos += 1;
        if *self
            .usage
            .as_bytes()
            .get(self.pos)
            .expect(INTERNAL_ERROR_MSG)
            == b'-'
        {
            self.pos += 1;
            self.long(arg);
            return;
        }
        self.short(arg)
    }

    fn long(&mut self, arg: &mut Arg<'help>) {
        debug!("UsageParser::long");
        self.stop_at(long_end);
        let name = &self.usage[self.start..self.pos];
        if !self.explicit_name_set {
            debug!("UsageParser::long: setting name...{}", name);
            arg.id = name.into();
            arg.name = name;
        }
        debug!("UsageParser::long: setting long...{}", name);
        arg.long = Some(name);
        self.prev = UsageToken::Long;
    }

    fn short(&mut self, arg: &mut Arg<'help>) {
        debug!("UsageParser::short");
        let start = &self.usage[self.pos..];
        let short = start.chars().next().expect(INTERNAL_ERROR_MSG);
        debug!("UsageParser::short: setting short...{}", short);
        arg.short = Some(short);
        if arg.name.is_empty() {
            // --long takes precedence but doesn't set self.explicit_name_set
            let name = &start[..short.len_utf8()];
            debug!("UsageParser::short: setting name...{}", name);
            arg.id = name.into();
            arg.name = name;
        }
        self.prev = UsageToken::Short;
    }

    // "something..."
    fn multiple(&mut self, arg: &mut Arg) {
        debug!("UsageParser::multiple");
        let mut dot_counter = 1;
        let start = self.pos;
        let mut bytes = self.usage[start..].bytes();
        while bytes.next() == Some(b'.') {
            dot_counter += 1;
            self.pos += 1;
            if dot_counter == 3 {
                debug!("UsageParser::multiple: setting multiple");
                arg.settings.set(ArgSettings::MultipleOccurrences);
                if arg.is_takes_value_set() {
                    arg.settings.set(ArgSettings::MultipleValues);
                    arg.settings.set(ArgSettings::UseValueDelimiter);
                    arg.val_delim.get_or_insert(',');
                }
                self.prev = UsageToken::Multiple;
                self.pos += 1;
                break;
            }
        }
    }

    fn help(&mut self, arg: &mut Arg<'help>) {
        debug!("UsageParser::help");
        self.stop_at(help_start);
        self.start = self.pos + 1;
        self.pos = self.usage.len() - 1;
        debug!(
            "UsageParser::help: setting help...{}",
            &self.usage[self.start..self.pos]
        );
        arg.help = Some(&self.usage[self.start..self.pos]);
        self.pos += 1; // Move to next byte to keep from thinking ending ' is a start
        self.prev = UsageToken::Help;
    }

    fn default(&mut self, arg: &mut Arg<'help>) {
        debug!(
            "UsageParser::default: from=\"{}\"",
            &self.usage[self.pos..self.usage.len()]
        );
        self.pos += 1; // Skip @
        self.stop_at(default_value_end); // Find first space after value
        debug!(
            "UsageParser::default: setting default...\"{}\"",
            &self.usage[self.start..self.pos]
        );
        arg.settings.set(ArgSettings::TakesValue);
        arg.default_vals = vec![std::ffi::OsStr::new(&self.usage[self.start..self.pos])];
        self.prev = UsageToken::Default;
    }
}

#[inline]
fn name_end(b: u8) -> bool {
    b != b']' && b != b'>'
}

#[inline]
fn token(b: u8) -> bool {
    b != b'\'' && b != b'.' && b != b'<' && b != b'[' && b != b'-' && b != b'@'
}

#[inline]
fn long_end(b: u8) -> bool {
    b != b'\'' && b != b'.' && b != b'<' && b != b'[' && b != b'=' && b != b' '
}

#[inline]
fn help_start(b: u8) -> bool {
    b != b'\''
}

#[inline]
fn default_value_end(b: u8) -> bool {
    b != b' '
}

#[cfg(test)]
mod test {
    #![allow(deprecated)]

    use crate::builder::{Arg, ArgSettings};

    #[allow(clippy::cognitive_complexity)]
    #[test]
    fn create_flag_usage() {
        let a = Arg::from_usage("[flag] -f 'some help info'");
        assert_eq!(a.name, "flag");
        assert_eq!(a.short.unwrap(), 'f');
        assert!(a.long.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(a.val_names.is_empty());

        let a = Arg::from_usage("[flag] --flag 'some help info'");
        assert_eq!(a.name, "flag");
        assert_eq!(a.long.unwrap(), "flag");
        assert!(a.short.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(a.val_names.is_empty());

        let a = Arg::from_usage("--flag 'some help info'");
        assert_eq!(a.name, "flag");
        assert_eq!(a.long.unwrap(), "flag");
        assert!(a.short.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(a.val_names.is_empty());

        let a = Arg::from_usage("[flag] -f --flag 'some help info'");
        assert_eq!(a.name, "flag");
        assert_eq!(a.short.unwrap(), 'f');
        assert_eq!(a.long.unwrap(), "flag");
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(a.val_names.is_empty());

        let a = Arg::from_usage("[flag] -f... 'some help info'");
        assert_eq!(a.name, "flag");
        assert_eq!(a.short.unwrap(), 'f');
        assert!(a.long.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(a.val_names.is_empty());

        let a = Arg::from_usage("[flag] -f --flag... 'some help info'");
        assert_eq!(a.name, "flag");
        assert_eq!(a.long.unwrap(), "flag");
        assert_eq!(a.short.unwrap(), 'f');
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(a.val_names.is_empty());

        let a = Arg::from_usage("-f --flag... 'some help info'");
        assert_eq!(a.name, "flag");
        assert_eq!(a.long.unwrap(), "flag");
        assert_eq!(a.short.unwrap(), 'f');
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(a.val_names.is_empty());

        let a = Arg::from_usage("--flags");
        assert_eq!(a.name, "flags");
        assert_eq!(a.long.unwrap(), "flags");
        assert!(a.val_names.is_empty());

        let a = Arg::from_usage("--flags...");
        assert_eq!(a.name, "flags");
        assert_eq!(a.long.unwrap(), "flags");
        assert!(a.is_multiple_occurrences_set());
        assert!(a.val_names.is_empty());

        let a = Arg::from_usage("[flags] -f");
        assert_eq!(a.name, "flags");
        assert_eq!(a.short.unwrap(), 'f');
        assert!(a.val_names.is_empty());

        let a = Arg::from_usage("[flags] -f...");
        assert_eq!(a.name, "flags");
        assert_eq!(a.short.unwrap(), 'f');
        assert!(a.is_multiple_occurrences_set());
        assert!(a.val_names.is_empty());

        let a = Arg::from_usage("-f 'some help info'");
        assert_eq!(a.name, "f");
        assert_eq!(a.short.unwrap(), 'f');
        assert!(a.long.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(a.val_names.is_empty());

        let a = Arg::from_usage("-f");
        assert_eq!(a.name, "f");
        assert_eq!(a.short.unwrap(), 'f');
        assert!(a.val_names.is_empty());

        let a = Arg::from_usage("-f...");
        assert_eq!(a.name, "f");
        assert_eq!(a.short.unwrap(), 'f');
        assert!(a.is_multiple_occurrences_set());
        assert!(a.val_names.is_empty());
    }

    #[test]
    fn create_option_usage0() {
        // Short only
        let a = Arg::from_usage("[option] -o [opt] 'some help info'");
        assert_eq!(a.name, "option");
        assert_eq!(a.short.unwrap(), 'o');
        assert!(a.long.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(!a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"opt"]);
    }

    #[test]
    fn create_option_usage1() {
        let a = Arg::from_usage("-o [opt] 'some help info'");
        assert_eq!(a.name, "o");
        assert_eq!(a.short.unwrap(), 'o');
        assert!(a.long.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(!a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"opt"]);
    }

    #[test]
    fn create_option_usage2() {
        let a = Arg::from_usage("<option> -o <opt> 'some help info'");
        assert_eq!(a.name, "option");
        assert_eq!(a.short.unwrap(), 'o');
        assert!(a.long.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"opt"]);
    }

    #[test]
    fn create_option_usage3() {
        let a = Arg::from_usage("-o <opt> 'some help info'");
        assert_eq!(a.name, "o");
        assert_eq!(a.short.unwrap(), 'o');
        assert!(a.long.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"opt"]);
    }

    #[test]
    fn create_option_usage4() {
        let a = Arg::from_usage("[option] -o [opt]... 'some help info'");
        assert_eq!(a.name, "option");
        assert_eq!(a.short.unwrap(), 'o');
        assert!(a.long.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(!a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"opt"]);
    }

    #[test]
    fn create_option_usage5() {
        let a = Arg::from_usage("[option]... -o [opt] 'some help info'");
        assert_eq!(a.name, "option");
        assert_eq!(a.short.unwrap(), 'o');
        assert!(a.long.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(!a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"opt"]);
    }

    #[test]
    fn create_option_usage6() {
        let a = Arg::from_usage("-o [opt]... 'some help info'");
        assert_eq!(a.name, "o");
        assert_eq!(a.short.unwrap(), 'o');
        assert!(a.long.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(!a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"opt"]);
    }

    #[test]
    fn create_option_usage7() {
        let a = Arg::from_usage("<option> -o <opt>... 'some help info'");
        assert_eq!(a.name, "option");
        assert_eq!(a.short.unwrap(), 'o');
        assert!(a.long.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"opt"]);
    }

    #[test]
    fn create_option_usage8() {
        let a = Arg::from_usage("<option>... -o <opt> 'some help info'");
        assert_eq!(a.name, "option");
        assert_eq!(a.short.unwrap(), 'o');
        assert!(a.long.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"opt"]);
    }

    #[test]
    fn create_option_usage9() {
        let a = Arg::from_usage("-o <opt>... 'some help info'");
        assert_eq!(a.name, "o");
        assert_eq!(a.short.unwrap(), 'o');
        assert!(a.long.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"opt"]);
    }

    #[test]
    fn create_option_usage_long1() {
        let a = Arg::from_usage("[option] --opt [opt] 'some help info'");
        assert_eq!(a.name, "option");
        assert_eq!(a.long.unwrap(), "opt");
        assert!(a.short.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(!a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"opt"]);
    }

    #[test]
    fn create_option_usage_long2() {
        let a = Arg::from_usage("--opt [option] 'some help info'");
        assert_eq!(a.name, "opt");
        assert_eq!(a.long.unwrap(), "opt");
        assert!(a.short.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(!a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"option"]);
    }

    #[test]
    fn create_option_usage_long3() {
        let a = Arg::from_usage("<option> --opt <opt> 'some help info'");
        assert_eq!(a.name, "option");
        assert_eq!(a.long.unwrap(), "opt");
        assert!(a.short.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"opt"]);
    }

    #[test]
    fn create_option_usage_long4() {
        let a = Arg::from_usage("--opt <option> 'some help info'");
        assert_eq!(a.name, "opt");
        assert_eq!(a.long.unwrap(), "opt");
        assert!(a.short.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"option"]);
    }

    #[test]
    fn create_option_usage_long5() {
        let a = Arg::from_usage("[option] --opt [opt]... 'some help info'");
        assert_eq!(a.name, "option");
        assert_eq!(a.long.unwrap(), "opt");
        assert!(a.short.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(!a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"opt"]);
    }

    #[test]
    fn create_option_usage_long6() {
        let a = Arg::from_usage("[option]... --opt [opt] 'some help info'");
        assert_eq!(a.name, "option");
        assert_eq!(a.long.unwrap(), "opt");
        assert!(a.short.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(!a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"opt"]);
    }

    #[test]
    fn create_option_usage_long7() {
        let a = Arg::from_usage("--opt [option]... 'some help info'");
        assert_eq!(a.name, "opt");
        assert_eq!(a.long.unwrap(), "opt");
        assert!(a.short.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(!a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"option"]);
    }

    #[test]
    fn create_option_usage_long8() {
        let a = Arg::from_usage("<option> --opt <opt>... 'some help info'");
        assert_eq!(a.name, "option");
        assert_eq!(a.long.unwrap(), "opt");
        assert!(a.short.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"opt"]);
    }

    #[test]
    fn create_option_usage_long9() {
        let a = Arg::from_usage("<option>... --opt <opt> 'some help info'");
        assert_eq!(a.name, "option");
        assert_eq!(a.long.unwrap(), "opt");
        assert!(a.short.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"opt"]);
    }

    #[test]
    fn create_option_usage_long10() {
        let a = Arg::from_usage("--opt <option>... 'some help info'");
        assert_eq!(a.name, "opt");
        assert_eq!(a.long.unwrap(), "opt");
        assert!(a.short.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"option"]);
    }

    #[test]
    fn create_option_usage_long_equals1() {
        let a = Arg::from_usage("[option] --opt=[opt] 'some help info'");
        assert_eq!(a.name, "option");
        assert_eq!(a.long.unwrap(), "opt");
        assert!(a.short.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(!a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"opt"]);
    }

    #[test]
    fn create_option_usage_long_equals2() {
        let a = Arg::from_usage("--opt=[option] 'some help info'");
        assert_eq!(a.name, "opt");
        assert_eq!(a.long.unwrap(), "opt");
        assert!(a.short.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(!a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"option"]);
    }

    #[test]
    fn create_option_usage_long_equals3() {
        let a = Arg::from_usage("<option> --opt=<opt> 'some help info'");
        assert_eq!(a.name, "option");
        assert_eq!(a.long.unwrap(), "opt");
        assert!(a.short.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"opt"]);
    }

    #[test]
    fn create_option_usage_long_equals4() {
        let a = Arg::from_usage("--opt=<option> 'some help info'");
        assert_eq!(a.name, "opt");
        assert_eq!(a.long.unwrap(), "opt");
        assert!(a.short.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"option"]);
    }

    #[test]
    fn create_option_usage_long_equals5() {
        let a = Arg::from_usage("[option] --opt=[opt]... 'some help info'");
        assert_eq!(a.name, "option");
        assert_eq!(a.long.unwrap(), "opt");
        assert!(a.short.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(!a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"opt"]);
    }

    #[test]
    fn create_option_usage_long_equals6() {
        let a = Arg::from_usage("[option]... --opt=[opt] 'some help info'");
        assert_eq!(a.name, "option");
        assert_eq!(a.long.unwrap(), "opt");
        assert!(a.short.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(!a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"opt"]);
    }

    #[test]
    fn create_option_usage_long_equals7() {
        let a = Arg::from_usage("--opt=[option]... 'some help info'");
        assert_eq!(a.name, "opt");
        assert_eq!(a.long.unwrap(), "opt");
        assert!(a.short.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(!a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"option"]);
    }

    #[test]
    fn create_option_usage_long_equals8() {
        let a = Arg::from_usage("<option> --opt=<opt>... 'some help info'");
        assert_eq!(a.name, "option");
        assert_eq!(a.long.unwrap(), "opt");
        assert!(a.short.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"opt"]);
    }

    #[test]
    fn create_option_usage_long_equals9() {
        let a = Arg::from_usage("<option>... --opt=<opt> 'some help info'");
        assert_eq!(a.name, "option");
        assert_eq!(a.long.unwrap(), "opt");
        assert!(a.short.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"opt"]);
    }

    #[test]
    fn create_option_usage_long_equals10() {
        let a = Arg::from_usage("--opt=<option>... 'some help info'");
        assert_eq!(a.name, "opt");
        assert_eq!(a.long.unwrap(), "opt");
        assert!(a.short.is_none());
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"option"]);
    }

    #[test]
    fn create_option_usage_both1() {
        let a = Arg::from_usage("[option] -o --opt [option] 'some help info'");
        assert_eq!(a.name, "option");
        assert_eq!(a.long.unwrap(), "opt");
        assert_eq!(a.short.unwrap(), 'o');
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(!a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"option"]);
    }

    #[test]
    fn create_option_usage_both2() {
        let a = Arg::from_usage("-o --opt [option] 'some help info'");
        assert_eq!(a.name, "opt");
        assert_eq!(a.long.unwrap(), "opt");
        assert_eq!(a.short.unwrap(), 'o');
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(!a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"option"]);
    }

    #[test]
    fn create_option_usage_both3() {
        let a = Arg::from_usage("<option> -o --opt <opt> 'some help info'");
        assert_eq!(a.name, "option");
        assert_eq!(a.long.unwrap(), "opt");
        assert_eq!(a.short.unwrap(), 'o');
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"opt"]);
    }

    #[test]
    fn create_option_usage_both4() {
        let a = Arg::from_usage("-o --opt <option> 'some help info'");
        assert_eq!(a.name, "opt");
        assert_eq!(a.long.unwrap(), "opt");
        assert_eq!(a.short.unwrap(), 'o');
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"option"]);
    }

    #[test]
    fn create_option_usage_both5() {
        let a = Arg::from_usage("[option]... -o --opt [option] 'some help info'");
        assert_eq!(a.name, "option");
        assert_eq!(a.long.unwrap(), "opt");
        assert_eq!(a.short.unwrap(), 'o');
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(!a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"option"]);
    }

    #[test]
    fn create_option_usage_both6() {
        let a = Arg::from_usage("-o --opt [option]... 'some help info'");
        assert_eq!(a.name, "opt");
        assert_eq!(a.long.unwrap(), "opt");
        assert_eq!(a.short.unwrap(), 'o');
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(!a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"option"]);
    }

    #[test]
    fn create_option_usage_both7() {
        let a = Arg::from_usage("<option>... -o --opt <opt> 'some help info'");
        assert_eq!(a.name, "option");
        assert_eq!(a.long.unwrap(), "opt");
        assert_eq!(a.short.unwrap(), 'o');
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"opt"]);
    }

    #[test]
    fn create_option_usage_both8() {
        let a = Arg::from_usage("-o --opt <option>... 'some help info'");
        assert_eq!(a.name, "opt");
        assert_eq!(a.long.unwrap(), "opt");
        assert_eq!(a.short.unwrap(), 'o');
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"option"]);
    }

    #[test]
    fn create_option_usage_both_equals1() {
        let a = Arg::from_usage("[option] -o --opt=[option] 'some help info'");
        assert_eq!(a.name, "option");
        assert_eq!(a.long.unwrap(), "opt");
        assert_eq!(a.short.unwrap(), 'o');
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(!a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"option"]);
    }

    #[test]
    fn create_option_usage_both_equals2() {
        let a = Arg::from_usage("-o --opt=[option] 'some help info'");
        assert_eq!(a.name, "opt");
        assert_eq!(a.long.unwrap(), "opt");
        assert_eq!(a.short.unwrap(), 'o');
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(!a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"option"]);
    }

    #[test]
    fn create_option_usage_both_equals3() {
        let a = Arg::from_usage("<option> -o --opt=<opt> 'some help info'");
        assert_eq!(a.name, "option");
        assert_eq!(a.long.unwrap(), "opt");
        assert_eq!(a.short.unwrap(), 'o');
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"opt"]);
    }

    #[test]
    fn create_option_usage_both_equals4() {
        let a = Arg::from_usage("-o --opt=<option> 'some help info'");
        assert_eq!(a.name, "opt");
        assert_eq!(a.long.unwrap(), "opt");
        assert_eq!(a.short.unwrap(), 'o');
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"option"]);
    }

    #[test]
    fn create_option_usage_both_equals5() {
        let a = Arg::from_usage("[option]... -o --opt=[option] 'some help info'");
        assert_eq!(a.name, "option");
        assert_eq!(a.long.unwrap(), "opt");
        assert_eq!(a.short.unwrap(), 'o');
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(!a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"option"]);
    }

    #[test]
    fn create_option_usage_both_equals6() {
        let a = Arg::from_usage("-o --opt=[option]... 'some help info'");
        assert_eq!(a.name, "opt");
        assert_eq!(a.long.unwrap(), "opt");
        assert_eq!(a.short.unwrap(), 'o');
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(!a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"option"]);
    }

    #[test]
    fn create_option_usage_both_equals7() {
        let a = Arg::from_usage("<option>... -o --opt=<opt> 'some help info'");
        assert_eq!(a.name, "option");
        assert_eq!(a.long.unwrap(), "opt");
        assert_eq!(a.short.unwrap(), 'o');
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"opt"]);
    }

    #[test]
    fn create_option_usage_both_equals8() {
        let a = Arg::from_usage("-o --opt=<option>... 'some help info'");
        assert_eq!(a.name, "opt");
        assert_eq!(a.long.unwrap(), "opt");
        assert_eq!(a.short.unwrap(), 'o');
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"option"]);
    }

    #[test]
    fn create_option_with_vals1() {
        let a = Arg::from_usage("-o <file> <mode> 'some help info'");
        assert_eq!(a.name, "o");
        assert!(a.long.is_none());
        assert_eq!(a.short.unwrap(), 'o');
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"file", &"mode"]);
    }

    #[test]
    fn create_option_with_vals2() {
        let a = Arg::from_usage("-o <file> <mode>... 'some help info'");
        assert_eq!(a.name, "o");
        assert!(a.long.is_none());
        assert_eq!(a.short.unwrap(), 'o');
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"file", &"mode"]);
    }

    #[test]
    fn create_option_with_vals3() {
        let a = Arg::from_usage("--opt <file> <mode>... 'some help info'");
        assert_eq!(a.name, "opt");
        assert!(a.short.is_none());
        assert_eq!(a.long.unwrap(), "opt");
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"file", &"mode"]);
    }

    #[test]
    fn create_option_with_vals4() {
        let a = Arg::from_usage("[myopt] --opt <file> <mode> 'some help info'");
        assert_eq!(a.name, "myopt");
        assert!(a.short.is_none());
        assert_eq!(a.long.unwrap(), "opt");
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(!a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"file", &"mode"]);
    }

    #[test]
    fn create_option_with_vals5() {
        let a = Arg::from_usage("--opt <file> <mode> 'some help info'");
        assert_eq!(a.name, "opt");
        assert!(a.short.is_none());
        assert_eq!(a.long.unwrap(), "opt");
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
    }

    #[test]
    fn create_positional_usage() {
        let a = Arg::from_usage("[pos] 'some help info'");
        assert_eq!(a.name, "pos");
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(!a.is_required_set());
        assert!(a.val_names.is_empty());
    }

    #[test]
    fn create_positional_usage0() {
        let a = Arg::from_usage("<pos> 'some help info'");
        assert_eq!(a.name, "pos");
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_required_set());
        assert!(a.val_names.is_empty());
    }

    #[test]
    fn pos_mult_help() {
        let a = Arg::from_usage("[pos]... 'some help info'");
        assert_eq!(a.name, "pos");
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(!a.is_required_set());
        assert!(a.val_names.is_empty());
    }

    #[test]
    fn pos_help_lit_single_quote() {
        let a = Arg::from_usage("[pos]... 'some help\' info'");
        assert_eq!(a.name, "pos");
        assert_eq!(a.help.unwrap(), "some help' info");
        assert!(a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(!a.is_required_set());
        assert!(a.val_names.is_empty());
    }

    #[test]
    fn pos_help_double_lit_single_quote() {
        let a = Arg::from_usage("[pos]... 'some \'help\' info'");
        assert_eq!(a.name, "pos");
        assert_eq!(a.help.unwrap(), "some 'help' info");
        assert!(a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(!a.is_required_set());
        assert!(a.val_names.is_empty());
    }

    #[test]
    fn pos_help_newline() {
        let a = Arg::from_usage(
            "[pos]... 'some help{n}\
             info'",
        );
        assert_eq!(a.name, "pos");
        assert_eq!(a.help.unwrap(), "some help{n}info");
        assert!(a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(!a.is_required_set());
        assert!(a.val_names.is_empty());
    }

    #[test]
    fn pos_help_newline_lit_sq() {
        let a = Arg::from_usage(
            "[pos]... 'some help\' stuff{n}\
             info'",
        );
        assert_eq!(a.name, "pos");
        assert_eq!(a.help.unwrap(), "some help' stuff{n}info");
        assert!(a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(!a.is_required_set());
        assert!(a.val_names.is_empty());
    }

    #[test]
    fn pos_req_mult_help() {
        let a = Arg::from_usage("<pos>... 'some help info'");
        assert_eq!(a.name, "pos");
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_required_set());
        assert!(a.val_names.is_empty());
    }

    #[test]
    fn pos_req() {
        let a = Arg::from_usage("<pos>");
        assert_eq!(a.name, "pos");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_required_set());
        assert!(a.val_names.is_empty());
    }

    #[test]
    fn pos_mult() {
        let a = Arg::from_usage("[pos]...");
        assert_eq!(a.name, "pos");
        assert!(a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(!a.is_required_set());
        assert!(a.val_names.is_empty());
    }

    #[test]
    fn pos_req_mult_def_help() {
        let a = Arg::from_usage("<pos>... @a 'some help info'");
        assert_eq!(a.name, "pos");
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_required_set());
        assert!(a.val_names.is_empty());
        assert_eq!(a.default_vals, vec![std::ffi::OsStr::new("a")]);
    }

    #[test]
    fn create_option_with_vals1_def() {
        let a = Arg::from_usage("-o <file> <mode> @a 'some help info'");
        assert_eq!(a.name, "o");
        assert!(a.long.is_none());
        assert_eq!(a.short.unwrap(), 'o');
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"file", &"mode"]);
        assert_eq!(a.default_vals, vec![std::ffi::OsStr::new("a")]);
    }

    #[test]
    fn create_option_with_vals4_def() {
        let a = Arg::from_usage("[myopt] --opt <file> <mode> @a 'some help info'");
        assert_eq!(a.name, "myopt");
        assert!(a.short.is_none());
        assert_eq!(a.long.unwrap(), "opt");
        assert_eq!(a.help.unwrap(), "some help info");
        assert!(!a.is_multiple_occurrences_set());
        assert!(!a.is_multiple_values_set());
        assert!(a.is_takes_value_set());
        assert!(!a.is_required_set());
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"file", &"mode"]);
        assert_eq!(a.default_vals, vec![std::ffi::OsStr::new("a")]);
    }

    #[test]
    fn nonascii() {
        let a = Arg::from_usage("<ASCII> 'üñíčöĐ€'");
        assert_eq!(a.name, "ASCII");
        assert_eq!(a.help, Some("üñíčöĐ€"));
        let a = Arg::from_usage("<üñíčöĐ€> 'ASCII'");
        assert_eq!(a.name, "üñíčöĐ€");
        assert_eq!(a.help, Some("ASCII"));
        let a = Arg::from_usage("<üñíčöĐ€> 'üñíčöĐ€'");
        assert_eq!(a.name, "üñíčöĐ€");
        assert_eq!(a.help, Some("üñíčöĐ€"));
        let a = Arg::from_usage("-ø 'ø'");
        assert_eq!(a.name, "ø");
        assert_eq!(a.short, Some('ø'));
        assert_eq!(a.help, Some("ø"));
        let a = Arg::from_usage("--üñíčöĐ€ 'Nōṫ ASCII'");
        assert_eq!(a.name, "üñíčöĐ€");
        assert_eq!(a.long, Some("üñíčöĐ€"));
        assert_eq!(a.help, Some("Nōṫ ASCII"));
        let a = Arg::from_usage("[ñämê] --ôpt=[üñíčöĐ€] 'hælp'");
        assert_eq!(a.name, "ñämê");
        assert_eq!(a.long, Some("ôpt"));
        assert_eq!(a.val_names.iter().collect::<Vec<_>>(), [&"üñíčöĐ€"]);
        assert_eq!(a.help, Some("hælp"));
    }

    #[test]
    fn value_names_building_num_vals_from_usage() {
        use crate::Command;
        let m = Command::new("test")
            .arg(Arg::from_usage("--pos <who> <what> <why>"))
            .try_get_matches_from(vec!["myprog", "--pos", "val1", "val2", "val3"]);

        assert!(m.is_ok(), "{:?}", m.unwrap_err().kind());
        let m = m.unwrap();

        assert_eq!(
            m.values_of("pos").unwrap().collect::<Vec<_>>(),
            ["val1", "val2", "val3"]
        );
    }

    #[test]
    fn issue_665() {
        use crate::{error::ErrorKind, Command};
        // Verify fix for "arg_from_usage(): required values not being enforced when followed by another option"
        let res = Command::new("tester")
        .arg(Arg::from_usage("-v, --reroll-count=[N] 'Mark the patch series as PATCH vN'"))
        .arg(
            Arg::from_usage("--subject-prefix [Subject-Prefix] 'Use [Subject-Prefix] instead of the standard [PATCH] prefix'")
                .setting(ArgSettings::ForbidEmptyValues)
        )
        .try_get_matches_from(vec!["test", "--subject-prefix", "-v", "2"]);

        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind(), ErrorKind::EmptyValue);
    }
}
