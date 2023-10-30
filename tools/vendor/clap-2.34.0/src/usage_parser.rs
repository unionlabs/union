// Internal
use crate::{
    args::{settings::ArgSettings, Arg},
    map::VecMap,
    INTERNAL_ERROR_MSG,
};

#[derive(PartialEq, Debug)]
enum UsageToken {
    Name,
    ValName,
    Short,
    Long,
    Help,
    Multiple,
    Unknown,
}

#[doc(hidden)]
#[derive(Debug)]
pub struct UsageParser<'a> {
    usage: &'a str,
    pos: usize,
    start: usize,
    prev: UsageToken,
    explicit_name_set: bool,
}

impl<'a> UsageParser<'a> {
    fn new(usage: &'a str) -> Self {
        debugln!("UsageParser::new: usage={:?}", usage);
        UsageParser {
            usage,
            pos: 0,
            start: 0,
            prev: UsageToken::Unknown,
            explicit_name_set: false,
        }
    }

    pub fn from_usage(usage: &'a str) -> Self {
        debugln!("UsageParser::from_usage;");
        UsageParser::new(usage)
    }

    pub fn parse(mut self) -> Arg<'a, 'a> {
        debugln!("UsageParser::parse;");
        let mut arg = Arg::default();
        loop {
            debugln!("UsageParser::parse:iter: pos={};", self.pos);
            self.stop_at(token);
            if let Some(&c) = self.usage.as_bytes().get(self.pos) {
                match c {
                    b'-' => self.short_or_long(&mut arg),
                    b'.' => self.multiple(&mut arg),
                    b'\'' => self.help(&mut arg),
                    _ => self.name(&mut arg),
                }
            } else {
                break;
            }
        }
        debug_assert!(
            !arg.b.name.is_empty(),
            "No name found for Arg when parsing usage string: {}",
            self.usage
        );
        arg.v.num_vals = match arg.v.val_names {
            Some(ref v) if v.len() >= 2 => Some(v.len() as u64),
            _ => None,
        };
        debugln!("UsageParser::parse: vals...{:?}", arg.v.val_names);
        arg
    }

    fn name(&mut self, arg: &mut Arg<'a, 'a>) {
        debugln!("UsageParser::name;");
        if *self
            .usage
            .as_bytes()
            .get(self.pos)
            .expect(INTERNAL_ERROR_MSG)
            == b'<'
            && !self.explicit_name_set
        {
            arg.setb(ArgSettings::Required);
        }
        self.pos += 1;
        self.stop_at(name_end);
        let name = &self.usage[self.start..self.pos];
        if self.prev == UsageToken::Unknown {
            debugln!("UsageParser::name: setting name...{}", name);
            arg.b.name = name;
            if arg.s.long.is_none() && arg.s.short.is_none() {
                debugln!("UsageParser::name: explicit name set...");
                self.explicit_name_set = true;
                self.prev = UsageToken::Name;
            }
        } else {
            debugln!("UsageParser::name: setting val name...{}", name);
            if let Some(ref mut v) = arg.v.val_names {
                let len = v.len();
                v.insert(len, name);
            } else {
                let mut v = VecMap::new();
                v.insert(0, name);
                arg.v.val_names = Some(v);
                arg.setb(ArgSettings::TakesValue);
            }
            self.prev = UsageToken::ValName;
        }
    }

    fn stop_at<F>(&mut self, f: F)
    where
        F: Fn(u8) -> bool,
    {
        debugln!("UsageParser::stop_at;");
        self.start = self.pos;
        self.pos += self.usage[self.start..]
            .bytes()
            .take_while(|&b| f(b))
            .count();
    }

    fn short_or_long(&mut self, arg: &mut Arg<'a, 'a>) {
        debugln!("UsageParser::short_or_long;");
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

    fn long(&mut self, arg: &mut Arg<'a, 'a>) {
        debugln!("UsageParser::long;");
        self.stop_at(long_end);
        let name = &self.usage[self.start..self.pos];
        if !self.explicit_name_set {
            debugln!("UsageParser::long: setting name...{}", name);
            arg.b.name = name;
        }
        debugln!("UsageParser::long: setting long...{}", name);
        arg.s.long = Some(name);
        self.prev = UsageToken::Long;
    }

    fn short(&mut self, arg: &mut Arg<'a, 'a>) {
        debugln!("UsageParser::short;");
        let start = &self.usage[self.pos..];
        let short = start.chars().next().expect(INTERNAL_ERROR_MSG);
        debugln!("UsageParser::short: setting short...{}", short);
        arg.s.short = Some(short);
        if arg.b.name.is_empty() {
            // --long takes precedence but doesn't set self.explicit_name_set
            let name = &start[..short.len_utf8()];
            debugln!("UsageParser::short: setting name...{}", name);
            arg.b.name = name;
        }
        self.prev = UsageToken::Short;
    }

    // "something..."
    fn multiple(&mut self, arg: &mut Arg) {
        debugln!("UsageParser::multiple;");
        let mut dot_counter = 1;
        let start = self.pos;
        let mut bytes = self.usage[start..].bytes();
        while bytes.next() == Some(b'.') {
            dot_counter += 1;
            self.pos += 1;
            if dot_counter == 3 {
                debugln!("UsageParser::multiple: setting multiple");
                arg.setb(ArgSettings::Multiple);
                if arg.is_set(ArgSettings::TakesValue) {
                    arg.setb(ArgSettings::UseValueDelimiter);
                    arg.unsetb(ArgSettings::ValueDelimiterNotSet);
                    if arg.v.val_delim.is_none() {
                        arg.v.val_delim = Some(',');
                    }
                }
                self.prev = UsageToken::Multiple;
                self.pos += 1;
                break;
            }
        }
    }

    fn help(&mut self, arg: &mut Arg<'a, 'a>) {
        debugln!("UsageParser::help;");
        self.stop_at(help_start);
        self.start = self.pos + 1;
        self.pos = self.usage.len() - 1;
        debugln!(
            "UsageParser::help: setting help...{}",
            &self.usage[self.start..self.pos]
        );
        arg.b.help = Some(&self.usage[self.start..self.pos]);
        self.pos += 1; // Move to next byte to keep from thinking ending ' is a start
        self.prev = UsageToken::Help;
    }
}

#[inline]
fn name_end(b: u8) -> bool {
    b != b']' && b != b'>'
}

#[inline]
fn token(b: u8) -> bool {
    b != b'\'' && b != b'.' && b != b'<' && b != b'[' && b != b'-'
}

#[inline]
fn long_end(b: u8) -> bool {
    b != b'\'' && b != b'.' && b != b'<' && b != b'[' && b != b'=' && b != b' '
}

#[inline]
fn help_start(b: u8) -> bool {
    b != b'\''
}

#[cfg(test)]
mod test {
    use crate::args::{Arg, ArgSettings};

    #[test]
    fn create_flag_usage() {
        let a = Arg::from_usage("[flag] -f 'some help info'");
        assert_eq!(a.b.name, "flag");
        assert_eq!(a.s.short.unwrap(), 'f');
        assert!(a.s.long.is_none());
        assert_eq!(a.b.help.unwrap(), "some help info");
        assert!(!a.is_set(ArgSettings::Multiple));
        assert!(a.v.val_names.is_none());
        assert!(a.v.num_vals.is_none());

        let b = Arg::from_usage("[flag] --flag 'some help info'");
        assert_eq!(b.b.name, "flag");
        assert_eq!(b.s.long.unwrap(), "flag");
        assert!(b.s.short.is_none());
        assert_eq!(b.b.help.unwrap(), "some help info");
        assert!(!b.is_set(ArgSettings::Multiple));
        assert!(a.v.val_names.is_none());
        assert!(a.v.num_vals.is_none());

        let b = Arg::from_usage("--flag 'some help info'");
        assert_eq!(b.b.name, "flag");
        assert_eq!(b.s.long.unwrap(), "flag");
        assert!(b.s.short.is_none());
        assert_eq!(b.b.help.unwrap(), "some help info");
        assert!(!b.is_set(ArgSettings::Multiple));
        assert!(b.v.val_names.is_none());
        assert!(b.v.num_vals.is_none());

        let c = Arg::from_usage("[flag] -f --flag 'some help info'");
        assert_eq!(c.b.name, "flag");
        assert_eq!(c.s.short.unwrap(), 'f');
        assert_eq!(c.s.long.unwrap(), "flag");
        assert_eq!(c.b.help.unwrap(), "some help info");
        assert!(!c.is_set(ArgSettings::Multiple));
        assert!(c.v.val_names.is_none());
        assert!(c.v.num_vals.is_none());

        let d = Arg::from_usage("[flag] -f... 'some help info'");
        assert_eq!(d.b.name, "flag");
        assert_eq!(d.s.short.unwrap(), 'f');
        assert!(d.s.long.is_none());
        assert_eq!(d.b.help.unwrap(), "some help info");
        assert!(d.is_set(ArgSettings::Multiple));
        assert!(d.v.val_names.is_none());
        assert!(d.v.num_vals.is_none());

        let e = Arg::from_usage("[flag] -f --flag... 'some help info'");
        assert_eq!(e.b.name, "flag");
        assert_eq!(e.s.long.unwrap(), "flag");
        assert_eq!(e.s.short.unwrap(), 'f');
        assert_eq!(e.b.help.unwrap(), "some help info");
        assert!(e.is_set(ArgSettings::Multiple));
        assert!(e.v.val_names.is_none());
        assert!(e.v.num_vals.is_none());

        let e = Arg::from_usage("-f --flag... 'some help info'");
        assert_eq!(e.b.name, "flag");
        assert_eq!(e.s.long.unwrap(), "flag");
        assert_eq!(e.s.short.unwrap(), 'f');
        assert_eq!(e.b.help.unwrap(), "some help info");
        assert!(e.is_set(ArgSettings::Multiple));
        assert!(e.v.val_names.is_none());
        assert!(e.v.num_vals.is_none());

        let e = Arg::from_usage("--flags");
        assert_eq!(e.b.name, "flags");
        assert_eq!(e.s.long.unwrap(), "flags");
        assert!(e.v.val_names.is_none());
        assert!(e.v.num_vals.is_none());

        let e = Arg::from_usage("--flags...");
        assert_eq!(e.b.name, "flags");
        assert_eq!(e.s.long.unwrap(), "flags");
        assert!(e.is_set(ArgSettings::Multiple));
        assert!(e.v.val_names.is_none());
        assert!(e.v.num_vals.is_none());

        let e = Arg::from_usage("[flags] -f");
        assert_eq!(e.b.name, "flags");
        assert_eq!(e.s.short.unwrap(), 'f');
        assert!(e.v.val_names.is_none());
        assert!(e.v.num_vals.is_none());

        let e = Arg::from_usage("[flags] -f...");
        assert_eq!(e.b.name, "flags");
        assert_eq!(e.s.short.unwrap(), 'f');
        assert!(e.is_set(ArgSettings::Multiple));
        assert!(e.v.val_names.is_none());
        assert!(e.v.num_vals.is_none());

        let a = Arg::from_usage("-f 'some help info'");
        assert_eq!(a.b.name, "f");
        assert_eq!(a.s.short.unwrap(), 'f');
        assert!(a.s.long.is_none());
        assert_eq!(a.b.help.unwrap(), "some help info");
        assert!(!a.is_set(ArgSettings::Multiple));
        assert!(a.v.val_names.is_none());
        assert!(a.v.num_vals.is_none());

        let e = Arg::from_usage("-f");
        assert_eq!(e.b.name, "f");
        assert_eq!(e.s.short.unwrap(), 'f');
        assert!(e.v.val_names.is_none());
        assert!(e.v.num_vals.is_none());

        let e = Arg::from_usage("-f...");
        assert_eq!(e.b.name, "f");
        assert_eq!(e.s.short.unwrap(), 'f');
        assert!(e.is_set(ArgSettings::Multiple));
        assert!(e.v.val_names.is_none());
        assert!(e.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage0() {
        // Short only
        let a = Arg::from_usage("[option] -o [opt] 'some help info'");
        assert_eq!(a.b.name, "option");
        assert_eq!(a.s.short.unwrap(), 'o');
        assert!(a.s.long.is_none());
        assert_eq!(a.b.help.unwrap(), "some help info");
        assert!(!a.is_set(ArgSettings::Multiple));
        assert!(a.is_set(ArgSettings::TakesValue));
        assert!(!a.is_set(ArgSettings::Required));
        assert_eq!(
            a.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"opt"]
        );
        assert!(a.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage1() {
        let b = Arg::from_usage("-o [opt] 'some help info'");
        assert_eq!(b.b.name, "o");
        assert_eq!(b.s.short.unwrap(), 'o');
        assert!(b.s.long.is_none());
        assert_eq!(b.b.help.unwrap(), "some help info");
        assert!(!b.is_set(ArgSettings::Multiple));
        assert!(b.is_set(ArgSettings::TakesValue));
        assert!(!b.is_set(ArgSettings::Required));
        assert_eq!(
            b.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"opt"]
        );
        assert!(b.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage2() {
        let c = Arg::from_usage("<option> -o <opt> 'some help info'");
        assert_eq!(c.b.name, "option");
        assert_eq!(c.s.short.unwrap(), 'o');
        assert!(c.s.long.is_none());
        assert_eq!(c.b.help.unwrap(), "some help info");
        assert!(!c.is_set(ArgSettings::Multiple));
        assert!(c.is_set(ArgSettings::TakesValue));
        assert!(c.is_set(ArgSettings::Required));
        assert_eq!(
            c.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"opt"]
        );
        assert!(c.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage3() {
        let d = Arg::from_usage("-o <opt> 'some help info'");
        assert_eq!(d.b.name, "o");
        assert_eq!(d.s.short.unwrap(), 'o');
        assert!(d.s.long.is_none());
        assert_eq!(d.b.help.unwrap(), "some help info");
        assert!(!d.is_set(ArgSettings::Multiple));
        assert!(d.is_set(ArgSettings::TakesValue));
        assert!(d.is_set(ArgSettings::Required));
        assert_eq!(
            d.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"opt"]
        );
        assert!(d.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage4() {
        let a = Arg::from_usage("[option] -o [opt]... 'some help info'");
        assert_eq!(a.b.name, "option");
        assert_eq!(a.s.short.unwrap(), 'o');
        assert!(a.s.long.is_none());
        assert_eq!(a.b.help.unwrap(), "some help info");
        assert!(a.is_set(ArgSettings::Multiple));
        assert!(a.is_set(ArgSettings::TakesValue));
        assert!(!a.is_set(ArgSettings::Required));
        assert_eq!(
            a.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"opt"]
        );
        assert!(a.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage5() {
        let a = Arg::from_usage("[option]... -o [opt] 'some help info'");
        assert_eq!(a.b.name, "option");
        assert_eq!(a.s.short.unwrap(), 'o');
        assert!(a.s.long.is_none());
        assert_eq!(a.b.help.unwrap(), "some help info");
        assert!(a.is_set(ArgSettings::Multiple));
        assert!(a.is_set(ArgSettings::TakesValue));
        assert!(!a.is_set(ArgSettings::Required));
        assert_eq!(
            a.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"opt"]
        );
        assert!(a.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage6() {
        let b = Arg::from_usage("-o [opt]... 'some help info'");
        assert_eq!(b.b.name, "o");
        assert_eq!(b.s.short.unwrap(), 'o');
        assert!(b.s.long.is_none());
        assert_eq!(b.b.help.unwrap(), "some help info");
        assert!(b.is_set(ArgSettings::Multiple));
        assert!(b.is_set(ArgSettings::TakesValue));
        assert!(!b.is_set(ArgSettings::Required));
        assert_eq!(
            b.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"opt"]
        );
        assert!(b.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage7() {
        let c = Arg::from_usage("<option> -o <opt>... 'some help info'");
        assert_eq!(c.b.name, "option");
        assert_eq!(c.s.short.unwrap(), 'o');
        assert!(c.s.long.is_none());
        assert_eq!(c.b.help.unwrap(), "some help info");
        assert!(c.is_set(ArgSettings::Multiple));
        assert!(c.is_set(ArgSettings::TakesValue));
        assert!(c.is_set(ArgSettings::Required));
        assert_eq!(
            c.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"opt"]
        );
        assert!(c.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage8() {
        let c = Arg::from_usage("<option>... -o <opt> 'some help info'");
        assert_eq!(c.b.name, "option");
        assert_eq!(c.s.short.unwrap(), 'o');
        assert!(c.s.long.is_none());
        assert_eq!(c.b.help.unwrap(), "some help info");
        assert!(c.is_set(ArgSettings::Multiple));
        assert!(c.is_set(ArgSettings::TakesValue));
        assert!(c.is_set(ArgSettings::Required));
        assert_eq!(
            c.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"opt"]
        );
        assert!(c.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage9() {
        let d = Arg::from_usage("-o <opt>... 'some help info'");
        assert_eq!(d.b.name, "o");
        assert_eq!(d.s.short.unwrap(), 'o');
        assert!(d.s.long.is_none());
        assert_eq!(d.b.help.unwrap(), "some help info");
        assert!(d.is_set(ArgSettings::Multiple));
        assert!(d.is_set(ArgSettings::TakesValue));
        assert!(d.is_set(ArgSettings::Required));
        assert_eq!(
            d.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"opt"]
        );
        assert!(d.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_long1() {
        let a = Arg::from_usage("[option] --opt [opt] 'some help info'");
        assert_eq!(a.b.name, "option");
        assert_eq!(a.s.long.unwrap(), "opt");
        assert!(a.s.short.is_none());
        assert_eq!(a.b.help.unwrap(), "some help info");
        assert!(!a.is_set(ArgSettings::Multiple));
        assert!(a.is_set(ArgSettings::TakesValue));
        assert!(!a.is_set(ArgSettings::Required));
        assert_eq!(
            a.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"opt"]
        );
        assert!(a.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_long2() {
        let b = Arg::from_usage("--opt [option] 'some help info'");
        assert_eq!(b.b.name, "opt");
        assert_eq!(b.s.long.unwrap(), "opt");
        assert!(b.s.short.is_none());
        assert_eq!(b.b.help.unwrap(), "some help info");
        assert!(!b.is_set(ArgSettings::Multiple));
        assert!(b.is_set(ArgSettings::TakesValue));
        assert!(!b.is_set(ArgSettings::Required));
        assert_eq!(
            b.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"option"]
        );
        assert!(b.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_long3() {
        let c = Arg::from_usage("<option> --opt <opt> 'some help info'");
        assert_eq!(c.b.name, "option");
        assert_eq!(c.s.long.unwrap(), "opt");
        assert!(c.s.short.is_none());
        assert_eq!(c.b.help.unwrap(), "some help info");
        assert!(!c.is_set(ArgSettings::Multiple));
        assert!(c.is_set(ArgSettings::TakesValue));
        assert!(c.is_set(ArgSettings::Required));
        assert_eq!(
            c.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"opt"]
        );
        assert!(c.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_long4() {
        let d = Arg::from_usage("--opt <option> 'some help info'");
        assert_eq!(d.b.name, "opt");
        assert_eq!(d.s.long.unwrap(), "opt");
        assert!(d.s.short.is_none());
        assert_eq!(d.b.help.unwrap(), "some help info");
        assert!(!d.is_set(ArgSettings::Multiple));
        assert!(d.is_set(ArgSettings::TakesValue));
        assert!(d.is_set(ArgSettings::Required));
        assert_eq!(
            d.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"option"]
        );
        assert!(d.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_long5() {
        let a = Arg::from_usage("[option] --opt [opt]... 'some help info'");
        assert_eq!(a.b.name, "option");
        assert_eq!(a.s.long.unwrap(), "opt");
        assert!(a.s.short.is_none());
        assert_eq!(a.b.help.unwrap(), "some help info");
        assert!(a.is_set(ArgSettings::Multiple));
        assert!(a.is_set(ArgSettings::TakesValue));
        assert!(!a.is_set(ArgSettings::Required));
        assert_eq!(
            a.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"opt"]
        );
        assert!(a.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_long6() {
        let a = Arg::from_usage("[option]... --opt [opt] 'some help info'");
        assert_eq!(a.b.name, "option");
        assert_eq!(a.s.long.unwrap(), "opt");
        assert!(a.s.short.is_none());
        assert_eq!(a.b.help.unwrap(), "some help info");
        assert!(a.is_set(ArgSettings::Multiple));
        assert!(a.is_set(ArgSettings::TakesValue));
        assert!(!a.is_set(ArgSettings::Required));
        assert_eq!(
            a.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"opt"]
        );
        assert!(a.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_long7() {
        let b = Arg::from_usage("--opt [option]... 'some help info'");
        assert_eq!(b.b.name, "opt");
        assert_eq!(b.s.long.unwrap(), "opt");
        assert!(b.s.short.is_none());
        assert_eq!(b.b.help.unwrap(), "some help info");
        assert!(b.is_set(ArgSettings::Multiple));
        assert!(b.is_set(ArgSettings::TakesValue));
        assert!(!b.is_set(ArgSettings::Required));
        assert_eq!(
            b.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"option"]
        );
        assert!(b.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_long8() {
        let c = Arg::from_usage("<option> --opt <opt>... 'some help info'");
        assert_eq!(c.b.name, "option");
        assert_eq!(c.s.long.unwrap(), "opt");
        assert!(c.s.short.is_none());
        assert_eq!(c.b.help.unwrap(), "some help info");
        assert!(c.is_set(ArgSettings::Multiple));
        assert!(c.is_set(ArgSettings::TakesValue));
        assert!(c.is_set(ArgSettings::Required));
        assert_eq!(
            c.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"opt"]
        );
        assert!(c.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_long9() {
        let c = Arg::from_usage("<option>... --opt <opt> 'some help info'");
        assert_eq!(c.b.name, "option");
        assert_eq!(c.s.long.unwrap(), "opt");
        assert!(c.s.short.is_none());
        assert_eq!(c.b.help.unwrap(), "some help info");
        assert!(c.is_set(ArgSettings::Multiple));
        assert!(c.is_set(ArgSettings::TakesValue));
        assert!(c.is_set(ArgSettings::Required));
        assert_eq!(
            c.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"opt"]
        );
        assert!(c.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_long10() {
        let d = Arg::from_usage("--opt <option>... 'some help info'");
        assert_eq!(d.b.name, "opt");
        assert_eq!(d.s.long.unwrap(), "opt");
        assert!(d.s.short.is_none());
        assert_eq!(d.b.help.unwrap(), "some help info");
        assert!(d.is_set(ArgSettings::Multiple));
        assert!(d.is_set(ArgSettings::TakesValue));
        assert!(d.is_set(ArgSettings::Required));
        assert_eq!(
            d.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"option"]
        );
        assert!(d.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_long_equals1() {
        let a = Arg::from_usage("[option] --opt=[opt] 'some help info'");
        assert_eq!(a.b.name, "option");
        assert_eq!(a.s.long.unwrap(), "opt");
        assert!(a.s.short.is_none());
        assert_eq!(a.b.help.unwrap(), "some help info");
        assert!(!a.is_set(ArgSettings::Multiple));
        assert!(a.is_set(ArgSettings::TakesValue));
        assert!(!a.is_set(ArgSettings::Required));
        assert_eq!(
            a.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"opt"]
        );
        assert!(a.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_long_equals2() {
        let b = Arg::from_usage("--opt=[option] 'some help info'");
        assert_eq!(b.b.name, "opt");
        assert_eq!(b.s.long.unwrap(), "opt");
        assert!(b.s.short.is_none());
        assert_eq!(b.b.help.unwrap(), "some help info");
        assert!(!b.is_set(ArgSettings::Multiple));
        assert!(b.is_set(ArgSettings::TakesValue));
        assert!(!b.is_set(ArgSettings::Required));
        assert_eq!(
            b.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"option"]
        );
        assert!(b.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_long_equals3() {
        let c = Arg::from_usage("<option> --opt=<opt> 'some help info'");
        assert_eq!(c.b.name, "option");
        assert_eq!(c.s.long.unwrap(), "opt");
        assert!(c.s.short.is_none());
        assert_eq!(c.b.help.unwrap(), "some help info");
        assert!(!c.is_set(ArgSettings::Multiple));
        assert!(c.is_set(ArgSettings::TakesValue));
        assert!(c.is_set(ArgSettings::Required));
        assert_eq!(
            c.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"opt"]
        );
        assert!(c.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_long_equals4() {
        let d = Arg::from_usage("--opt=<option> 'some help info'");
        assert_eq!(d.b.name, "opt");
        assert_eq!(d.s.long.unwrap(), "opt");
        assert!(d.s.short.is_none());
        assert_eq!(d.b.help.unwrap(), "some help info");
        assert!(!d.is_set(ArgSettings::Multiple));
        assert!(d.is_set(ArgSettings::TakesValue));
        assert!(d.is_set(ArgSettings::Required));
        assert_eq!(
            d.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"option"]
        );
        assert!(d.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_long_equals5() {
        let a = Arg::from_usage("[option] --opt=[opt]... 'some help info'");
        assert_eq!(a.b.name, "option");
        assert_eq!(a.s.long.unwrap(), "opt");
        assert!(a.s.short.is_none());
        assert_eq!(a.b.help.unwrap(), "some help info");
        assert!(a.is_set(ArgSettings::Multiple));
        assert!(a.is_set(ArgSettings::TakesValue));
        assert!(!a.is_set(ArgSettings::Required));
        assert_eq!(
            a.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"opt"]
        );
        assert!(a.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_long_equals6() {
        let a = Arg::from_usage("[option]... --opt=[opt] 'some help info'");
        assert_eq!(a.b.name, "option");
        assert_eq!(a.s.long.unwrap(), "opt");
        assert!(a.s.short.is_none());
        assert_eq!(a.b.help.unwrap(), "some help info");
        assert!(a.is_set(ArgSettings::Multiple));
        assert!(a.is_set(ArgSettings::TakesValue));
        assert!(!a.is_set(ArgSettings::Required));
        assert_eq!(
            a.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"opt"]
        );
        assert!(a.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_long_equals7() {
        let b = Arg::from_usage("--opt=[option]... 'some help info'");
        assert_eq!(b.b.name, "opt");
        assert_eq!(b.s.long.unwrap(), "opt");
        assert!(b.s.short.is_none());
        assert_eq!(b.b.help.unwrap(), "some help info");
        assert!(b.is_set(ArgSettings::Multiple));
        assert!(b.is_set(ArgSettings::TakesValue));
        assert!(!b.is_set(ArgSettings::Required));
        assert_eq!(
            b.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"option"]
        );
        assert!(b.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_long_equals8() {
        let c = Arg::from_usage("<option> --opt=<opt>... 'some help info'");
        assert_eq!(c.b.name, "option");
        assert_eq!(c.s.long.unwrap(), "opt");
        assert!(c.s.short.is_none());
        assert_eq!(c.b.help.unwrap(), "some help info");
        assert!(c.is_set(ArgSettings::Multiple));
        assert!(c.is_set(ArgSettings::TakesValue));
        assert!(c.is_set(ArgSettings::Required));
        assert_eq!(
            c.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"opt"]
        );
        assert!(c.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_long_equals9() {
        let c = Arg::from_usage("<option>... --opt=<opt> 'some help info'");
        assert_eq!(c.b.name, "option");
        assert_eq!(c.s.long.unwrap(), "opt");
        assert!(c.s.short.is_none());
        assert_eq!(c.b.help.unwrap(), "some help info");
        assert!(c.is_set(ArgSettings::Multiple));
        assert!(c.is_set(ArgSettings::TakesValue));
        assert!(c.is_set(ArgSettings::Required));
        assert_eq!(
            c.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"opt"]
        );
        assert!(c.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_long_equals10() {
        let d = Arg::from_usage("--opt=<option>... 'some help info'");
        assert_eq!(d.b.name, "opt");
        assert_eq!(d.s.long.unwrap(), "opt");
        assert!(d.s.short.is_none());
        assert_eq!(d.b.help.unwrap(), "some help info");
        assert!(d.is_set(ArgSettings::Multiple));
        assert!(d.is_set(ArgSettings::TakesValue));
        assert!(d.is_set(ArgSettings::Required));
        assert_eq!(
            d.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"option"]
        );
        assert!(d.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_both1() {
        let a = Arg::from_usage("[option] -o --opt [option] 'some help info'");
        assert_eq!(a.b.name, "option");
        assert_eq!(a.s.long.unwrap(), "opt");
        assert_eq!(a.s.short.unwrap(), 'o');
        assert_eq!(a.b.help.unwrap(), "some help info");
        assert!(!a.is_set(ArgSettings::Multiple));
        assert!(a.is_set(ArgSettings::TakesValue));
        assert!(!a.is_set(ArgSettings::Required));
        assert_eq!(
            a.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"option"]
        );
        assert!(a.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_both2() {
        let b = Arg::from_usage("-o --opt [option] 'some help info'");
        assert_eq!(b.b.name, "opt");
        assert_eq!(b.s.long.unwrap(), "opt");
        assert_eq!(b.s.short.unwrap(), 'o');
        assert_eq!(b.b.help.unwrap(), "some help info");
        assert!(!b.is_set(ArgSettings::Multiple));
        assert!(b.is_set(ArgSettings::TakesValue));
        assert!(!b.is_set(ArgSettings::Required));
        assert_eq!(
            b.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"option"]
        );
        assert!(b.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_both3() {
        let c = Arg::from_usage("<option> -o --opt <opt> 'some help info'");
        assert_eq!(c.b.name, "option");
        assert_eq!(c.s.long.unwrap(), "opt");
        assert_eq!(c.s.short.unwrap(), 'o');
        assert_eq!(c.b.help.unwrap(), "some help info");
        assert!(!c.is_set(ArgSettings::Multiple));
        assert!(c.is_set(ArgSettings::TakesValue));
        assert!(c.is_set(ArgSettings::Required));
        assert_eq!(
            c.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"opt"]
        );
        assert!(c.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_both4() {
        let d = Arg::from_usage("-o --opt <option> 'some help info'");
        assert_eq!(d.b.name, "opt");
        assert_eq!(d.s.long.unwrap(), "opt");
        assert_eq!(d.s.short.unwrap(), 'o');
        assert_eq!(d.b.help.unwrap(), "some help info");
        assert!(!d.is_set(ArgSettings::Multiple));
        assert!(d.is_set(ArgSettings::TakesValue));
        assert!(d.is_set(ArgSettings::Required));
        assert_eq!(
            d.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"option"]
        );
        assert!(d.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_both5() {
        let a = Arg::from_usage("[option]... -o --opt [option] 'some help info'");
        assert_eq!(a.b.name, "option");
        assert_eq!(a.s.long.unwrap(), "opt");
        assert_eq!(a.s.short.unwrap(), 'o');
        assert_eq!(a.b.help.unwrap(), "some help info");
        assert!(a.is_set(ArgSettings::Multiple));
        assert!(a.is_set(ArgSettings::TakesValue));
        assert!(!a.is_set(ArgSettings::Required));
        assert_eq!(
            a.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"option"]
        );
        assert!(a.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_both6() {
        let b = Arg::from_usage("-o --opt [option]... 'some help info'");
        assert_eq!(b.b.name, "opt");
        assert_eq!(b.s.long.unwrap(), "opt");
        assert_eq!(b.s.short.unwrap(), 'o');
        assert_eq!(b.b.help.unwrap(), "some help info");
        assert!(b.is_set(ArgSettings::Multiple));
        assert!(b.is_set(ArgSettings::TakesValue));
        assert!(!b.is_set(ArgSettings::Required));
        assert_eq!(
            b.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"option"]
        );
        assert!(b.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_both7() {
        let c = Arg::from_usage("<option>... -o --opt <opt> 'some help info'");
        assert_eq!(c.b.name, "option");
        assert_eq!(c.s.long.unwrap(), "opt");
        assert_eq!(c.s.short.unwrap(), 'o');
        assert_eq!(c.b.help.unwrap(), "some help info");
        assert!(c.is_set(ArgSettings::Multiple));
        assert!(c.is_set(ArgSettings::TakesValue));
        assert!(c.is_set(ArgSettings::Required));
        assert_eq!(
            c.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"opt"]
        );
        assert!(c.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_both8() {
        let d = Arg::from_usage("-o --opt <option>... 'some help info'");
        assert_eq!(d.b.name, "opt");
        assert_eq!(d.s.long.unwrap(), "opt");
        assert_eq!(d.s.short.unwrap(), 'o');
        assert_eq!(d.b.help.unwrap(), "some help info");
        assert!(d.is_set(ArgSettings::Multiple));
        assert!(d.is_set(ArgSettings::TakesValue));
        assert!(d.is_set(ArgSettings::Required));
        assert_eq!(
            d.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"option"]
        );
        assert!(d.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_both_equals1() {
        let a = Arg::from_usage("[option] -o --opt=[option] 'some help info'");
        assert_eq!(a.b.name, "option");
        assert_eq!(a.s.long.unwrap(), "opt");
        assert_eq!(a.s.short.unwrap(), 'o');
        assert_eq!(a.b.help.unwrap(), "some help info");
        assert!(!a.is_set(ArgSettings::Multiple));
        assert!(a.is_set(ArgSettings::TakesValue));
        assert!(!a.is_set(ArgSettings::Required));
        assert_eq!(
            a.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"option"]
        );
        assert!(a.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_both_equals2() {
        let b = Arg::from_usage("-o --opt=[option] 'some help info'");
        assert_eq!(b.b.name, "opt");
        assert_eq!(b.s.long.unwrap(), "opt");
        assert_eq!(b.s.short.unwrap(), 'o');
        assert_eq!(b.b.help.unwrap(), "some help info");
        assert!(!b.is_set(ArgSettings::Multiple));
        assert!(b.is_set(ArgSettings::TakesValue));
        assert!(!b.is_set(ArgSettings::Required));
        assert_eq!(
            b.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"option"]
        );
        assert!(b.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_both_equals3() {
        let c = Arg::from_usage("<option> -o --opt=<opt> 'some help info'");
        assert_eq!(c.b.name, "option");
        assert_eq!(c.s.long.unwrap(), "opt");
        assert_eq!(c.s.short.unwrap(), 'o');
        assert_eq!(c.b.help.unwrap(), "some help info");
        assert!(!c.is_set(ArgSettings::Multiple));
        assert!(c.is_set(ArgSettings::TakesValue));
        assert!(c.is_set(ArgSettings::Required));
        assert_eq!(
            c.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"opt"]
        );
        assert!(c.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_both_equals4() {
        let d = Arg::from_usage("-o --opt=<option> 'some help info'");
        assert_eq!(d.b.name, "opt");
        assert_eq!(d.s.long.unwrap(), "opt");
        assert_eq!(d.s.short.unwrap(), 'o');
        assert_eq!(d.b.help.unwrap(), "some help info");
        assert!(!d.is_set(ArgSettings::Multiple));
        assert!(d.is_set(ArgSettings::TakesValue));
        assert!(d.is_set(ArgSettings::Required));
        assert_eq!(
            d.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"option"]
        );
        assert!(d.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_both_equals5() {
        let a = Arg::from_usage("[option]... -o --opt=[option] 'some help info'");
        assert_eq!(a.b.name, "option");
        assert_eq!(a.s.long.unwrap(), "opt");
        assert_eq!(a.s.short.unwrap(), 'o');
        assert_eq!(a.b.help.unwrap(), "some help info");
        assert!(a.is_set(ArgSettings::Multiple));
        assert!(a.is_set(ArgSettings::TakesValue));
        assert!(!a.is_set(ArgSettings::Required));
        assert_eq!(
            a.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"option"]
        );
        assert!(a.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_both_equals6() {
        let b = Arg::from_usage("-o --opt=[option]... 'some help info'");
        assert_eq!(b.b.name, "opt");
        assert_eq!(b.s.long.unwrap(), "opt");
        assert_eq!(b.s.short.unwrap(), 'o');
        assert_eq!(b.b.help.unwrap(), "some help info");
        assert!(b.is_set(ArgSettings::Multiple));
        assert!(b.is_set(ArgSettings::TakesValue));
        assert!(!b.is_set(ArgSettings::Required));
        assert_eq!(
            b.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"option"]
        );
        assert!(b.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_both_equals7() {
        let c = Arg::from_usage("<option>... -o --opt=<opt> 'some help info'");
        assert_eq!(c.b.name, "option");
        assert_eq!(c.s.long.unwrap(), "opt");
        assert_eq!(c.s.short.unwrap(), 'o');
        assert_eq!(c.b.help.unwrap(), "some help info");
        assert!(c.is_set(ArgSettings::Multiple));
        assert!(c.is_set(ArgSettings::TakesValue));
        assert!(c.is_set(ArgSettings::Required));
        assert_eq!(
            c.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"opt"]
        );
        assert!(c.v.num_vals.is_none());
    }

    #[test]
    fn create_option_usage_both_equals8() {
        let d = Arg::from_usage("-o --opt=<option>... 'some help info'");
        assert_eq!(d.b.name, "opt");
        assert_eq!(d.s.long.unwrap(), "opt");
        assert_eq!(d.s.short.unwrap(), 'o');
        assert_eq!(d.b.help.unwrap(), "some help info");
        assert!(d.is_set(ArgSettings::Multiple));
        assert!(d.is_set(ArgSettings::TakesValue));
        assert!(d.is_set(ArgSettings::Required));
        assert_eq!(
            d.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"option"]
        );
        assert!(d.v.num_vals.is_none());
    }

    #[test]
    fn create_option_with_vals1() {
        let d = Arg::from_usage("-o <file> <mode> 'some help info'");
        assert_eq!(d.b.name, "o");
        assert!(d.s.long.is_none());
        assert_eq!(d.s.short.unwrap(), 'o');
        assert_eq!(d.b.help.unwrap(), "some help info");
        assert!(!d.is_set(ArgSettings::Multiple));
        assert!(d.is_set(ArgSettings::TakesValue));
        assert!(d.is_set(ArgSettings::Required));
        assert_eq!(
            d.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"file", &"mode"]
        );
        assert_eq!(d.v.num_vals.unwrap(), 2);
    }

    #[test]
    fn create_option_with_vals2() {
        let d = Arg::from_usage("-o <file> <mode>... 'some help info'");
        assert_eq!(d.b.name, "o");
        assert!(d.s.long.is_none());
        assert_eq!(d.s.short.unwrap(), 'o');
        assert_eq!(d.b.help.unwrap(), "some help info");
        assert!(d.is_set(ArgSettings::Multiple));
        assert!(d.is_set(ArgSettings::TakesValue));
        assert!(d.is_set(ArgSettings::Required));
        assert_eq!(
            d.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"file", &"mode"]
        );
        assert_eq!(d.v.num_vals.unwrap(), 2);
    }

    #[test]
    fn create_option_with_vals3() {
        let d = Arg::from_usage("--opt <file> <mode>... 'some help info'");
        assert_eq!(d.b.name, "opt");
        assert!(d.s.short.is_none());
        assert_eq!(d.s.long.unwrap(), "opt");
        assert_eq!(d.b.help.unwrap(), "some help info");
        assert!(d.is_set(ArgSettings::Multiple));
        assert!(d.is_set(ArgSettings::TakesValue));
        assert!(d.is_set(ArgSettings::Required));
        assert_eq!(
            d.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"file", &"mode"]
        );
        assert_eq!(d.v.num_vals.unwrap(), 2);
    }

    #[test]
    fn create_option_with_vals4() {
        let d = Arg::from_usage("[myopt] --opt <file> <mode> 'some help info'");
        assert_eq!(d.b.name, "myopt");
        assert!(d.s.short.is_none());
        assert_eq!(d.s.long.unwrap(), "opt");
        assert_eq!(d.b.help.unwrap(), "some help info");
        assert!(!d.is_set(ArgSettings::Multiple));
        assert!(d.is_set(ArgSettings::TakesValue));
        assert!(!d.is_set(ArgSettings::Required));
        assert_eq!(
            d.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"file", &"mode"]
        );
        assert_eq!(d.v.num_vals.unwrap(), 2);
    }

    #[test]
    fn create_option_with_vals5() {
        let d = Arg::from_usage("--opt <file> <mode> 'some help info'");
        assert_eq!(d.b.name, "opt");
        assert!(d.s.short.is_none());
        assert_eq!(d.s.long.unwrap(), "opt");
        assert_eq!(d.b.help.unwrap(), "some help info");
        assert!(!d.is_set(ArgSettings::Multiple));
        assert!(d.is_set(ArgSettings::TakesValue));
        assert!(d.is_set(ArgSettings::Required));
        assert_eq!(d.v.num_vals.unwrap(), 2);
    }

    #[test]
    fn create_positional_usage() {
        let a = Arg::from_usage("[pos] 'some help info'");
        assert_eq!(a.b.name, "pos");
        assert_eq!(a.b.help.unwrap(), "some help info");
        assert!(!a.is_set(ArgSettings::Multiple));
        assert!(!a.is_set(ArgSettings::Required));
        assert!(a.v.val_names.is_none());
        assert!(a.v.num_vals.is_none());
    }

    #[test]
    fn create_positional_usage0() {
        let b = Arg::from_usage("<pos> 'some help info'");
        assert_eq!(b.b.name, "pos");
        assert_eq!(b.b.help.unwrap(), "some help info");
        assert!(!b.is_set(ArgSettings::Multiple));
        assert!(b.is_set(ArgSettings::Required));
        assert!(b.v.val_names.is_none());
        assert!(b.v.num_vals.is_none());
    }

    #[test]
    fn pos_mult_help() {
        let c = Arg::from_usage("[pos]... 'some help info'");
        assert_eq!(c.b.name, "pos");
        assert_eq!(c.b.help.unwrap(), "some help info");
        assert!(c.is_set(ArgSettings::Multiple));
        assert!(!c.is_set(ArgSettings::Required));
        assert!(c.v.val_names.is_none());
        assert!(c.v.num_vals.is_none());
    }

    #[test]
    fn pos_help_lit_single_quote() {
        let c = Arg::from_usage("[pos]... 'some help\' info'");
        assert_eq!(c.b.name, "pos");
        assert_eq!(c.b.help.unwrap(), "some help' info");
        assert!(c.is_set(ArgSettings::Multiple));
        assert!(!c.is_set(ArgSettings::Required));
        assert!(c.v.val_names.is_none());
        assert!(c.v.num_vals.is_none());
    }

    #[test]
    fn pos_help_double_lit_single_quote() {
        let c = Arg::from_usage("[pos]... 'some \'help\' info'");
        assert_eq!(c.b.name, "pos");
        assert_eq!(c.b.help.unwrap(), "some 'help' info");
        assert!(c.is_set(ArgSettings::Multiple));
        assert!(!c.is_set(ArgSettings::Required));
        assert!(c.v.val_names.is_none());
        assert!(c.v.num_vals.is_none());
    }

    #[test]
    fn pos_help_newline() {
        let c = Arg::from_usage(
            "[pos]... 'some help{n}\
             info'",
        );
        assert_eq!(c.b.name, "pos");
        assert_eq!(c.b.help.unwrap(), "some help{n}info");
        assert!(c.is_set(ArgSettings::Multiple));
        assert!(!c.is_set(ArgSettings::Required));
        assert!(c.v.val_names.is_none());
        assert!(c.v.num_vals.is_none());
    }

    #[test]
    fn pos_help_newline_lit_sq() {
        let c = Arg::from_usage(
            "[pos]... 'some help\' stuff{n}\
             info'",
        );
        assert_eq!(c.b.name, "pos");
        assert_eq!(c.b.help.unwrap(), "some help' stuff{n}info");
        assert!(c.is_set(ArgSettings::Multiple));
        assert!(!c.is_set(ArgSettings::Required));
        assert!(c.v.val_names.is_none());
        assert!(c.v.num_vals.is_none());
    }

    #[test]
    fn pos_req_mult_help() {
        let d = Arg::from_usage("<pos>... 'some help info'");
        assert_eq!(d.b.name, "pos");
        assert_eq!(d.b.help.unwrap(), "some help info");
        assert!(d.is_set(ArgSettings::Multiple));
        assert!(d.is_set(ArgSettings::Required));
        assert!(d.v.val_names.is_none());
        assert!(d.v.num_vals.is_none());
    }

    #[test]
    fn pos_req() {
        let b = Arg::from_usage("<pos>");
        assert_eq!(b.b.name, "pos");
        assert!(!b.is_set(ArgSettings::Multiple));
        assert!(b.is_set(ArgSettings::Required));
        assert!(b.v.val_names.is_none());
        assert!(b.v.num_vals.is_none());
    }

    #[test]
    fn pos_mult() {
        let c = Arg::from_usage("[pos]...");
        assert_eq!(c.b.name, "pos");
        assert!(c.is_set(ArgSettings::Multiple));
        assert!(!c.is_set(ArgSettings::Required));
        assert!(c.v.val_names.is_none());
        assert!(c.v.num_vals.is_none());
    }

    #[test]
    fn nonascii() {
        let a = Arg::from_usage("<ASCII> 'üñíčöĐ€'");
        assert_eq!(a.b.name, "ASCII");
        assert_eq!(a.b.help, Some("üñíčöĐ€"));
        let a = Arg::from_usage("<üñíčöĐ€> 'ASCII'");
        assert_eq!(a.b.name, "üñíčöĐ€");
        assert_eq!(a.b.help, Some("ASCII"));
        let a = Arg::from_usage("<üñíčöĐ€> 'üñíčöĐ€'");
        assert_eq!(a.b.name, "üñíčöĐ€");
        assert_eq!(a.b.help, Some("üñíčöĐ€"));
        let a = Arg::from_usage("-ø 'ø'");
        assert_eq!(a.b.name, "ø");
        assert_eq!(a.s.short, Some('ø'));
        assert_eq!(a.b.help, Some("ø"));
        let a = Arg::from_usage("--üñíčöĐ€ 'Nōṫ ASCII'");
        assert_eq!(a.b.name, "üñíčöĐ€");
        assert_eq!(a.s.long, Some("üñíčöĐ€"));
        assert_eq!(a.b.help, Some("Nōṫ ASCII"));
        let a = Arg::from_usage("[ñämê] --ôpt=[üñíčöĐ€] 'hælp'");
        assert_eq!(a.b.name, "ñämê");
        assert_eq!(a.s.long, Some("ôpt"));
        assert_eq!(
            a.v.val_names.unwrap().values().collect::<Vec<_>>(),
            [&"üñíčöĐ€"]
        );
        assert_eq!(a.b.help, Some("hælp"));
    }
}
