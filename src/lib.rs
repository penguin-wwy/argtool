use std::string::String;
use std::option::Option;
use std::clone::Clone;
use std::ffi::OsStr;
use std::iter::{IntoIterator, repeat};
use std::cmp::{Eq, PartialEq};
use std::iter::Iterator;

mod error;
use error::{Res, Fail};

mod test;

#[derive(Debug, Clone, Eq, PartialEq)]
enum Name {
    Long(String),
    Short(char),
}

impl Name {
    fn from_str(name: &str) -> Name {
        if name.len() == 1 {
            Name::Short(name.as_bytes()[0] as char)
        } else {
            Name::Long(name.to_string())
        }
    }

    fn to_string(&self) -> String {
        match *self {
            Name::Short(ch) => ch.to_string(),
            Name::Long(ref s) => s.to_string()
        }
    }
}

#[derive(Clone, Copy)]
pub enum HasArg {
    YES,
    NO,
    May,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Occur {
    Opt,   // 0 or 1
    Once,  // 1
    Multi,  // 0 or more times
}

#[derive(Clone)]
struct Argument {
    short_name: Option<Name>,
    long_name: Option<Name>,
    hint: String,
    desc: String,
    hasarg: HasArg,
    occur: Occur,
}

impl Argument {
    pub fn has_arg(&self) -> HasArg {
        self.hasarg
    }

    pub fn occur(&self) -> Occur {
        self.occur
    }

    pub fn transform(&self) -> Arg {
        Arg {
            value: Vec::new(),
            hasarg: self.has_arg(),
            occur: self.occur(),
            occured: 0,
            submem: Vec::new(),
            argrc: &self,
        }
    }
}

enum ParseStyle {
    StrictStyle,
    FreeStyle,
}

pub struct OptParser {
    args: Vec<Argument>,
    style: ParseStyle,
}

impl OptParser {
    pub fn new() -> Self {
        OptParser {
            args: Vec::new(),
            style: ParseStyle::FreeStyle,
        }
    }

    fn validate_names(short_name: &str, long_name: &str) {
        let len = short_name.len();
        assert!(len == 1 || len == 0,
                "the short_name (first argument) should be a single character, \
             or an empty string for none");
        let len = long_name.len();
        assert!(len == 0 || len > 1,
                "the long_name (second argument) should be longer than a single \
             character, or an empty string for none");
    }

    fn is_arg(arg: &str) -> bool {
        arg.as_bytes().get(0) == Some(&b'-') && arg.len() > 1
    }

    fn find_opt(&self, name: &Name) -> Option<usize> {

        let pos = self.args.iter().
            position(|opt| opt.long_name.clone().as_ref() == Some(name)
                || opt.short_name.clone().as_ref() == Some(name));
        if pos.is_some() {
            return pos;
        }

        None
    }

    pub fn choose_strict_style(&mut self) -> &mut Self {
        self.style = ParseStyle::StrictStyle;
        return self;
    }

    pub fn choose_free_style(&mut self) -> &mut Self {
        self.style = ParseStyle::FreeStyle;
        return self;
    }

    pub fn add_argument(&mut self, short_name: &str, long_name: &str,
                    hint: &str, desc: &str, hasarg: HasArg, occur: Occur) -> &mut Self {
        OptParser::validate_names(short_name, long_name);
        self.args.push(Argument {
            short_name: match short_name {
                "" => None,
                n => Some(Name::Short(short_name.chars().next().unwrap())),
            },
            long_name: match long_name {
                "" => None,
                n => Some(Name::from_str(long_name)),
            },
            hint: String::from(hint),
            desc: String::from(desc),
            hasarg: hasarg,
            occur: occur,
        });
        return self;
    }

    pub fn add_multi_arg(&mut self, short: &str, long: &str, desc: &str, hint: &str) -> &mut Self {
        self.add_argument(short, long, hint, desc, HasArg::YES, Occur::Multi)
    }

    pub fn add_necessary_arg(&mut self, short: &str, long: &str, desc: &str, hint: &str) -> &mut Self {
        self.add_argument(short, long, hint, desc, HasArg::YES, Occur::Once)
    }

    pub fn add_optional_arg(&mut self, short: &str, long: &str, desc: &str, hint: &str) -> &mut Self {
        self.add_argument(short, long, hint, desc, HasArg::YES, Occur::Opt)
    }

    pub fn add_necessary_flag(&mut self, short: &str, long: &str, desc: &str) -> &mut Self {
        self.add_argument(short, long, "", desc, HasArg::NO, Occur::Once)
    }

    pub fn add_optional_flag(&mut self, short: &str, long: &str, desc: &str) -> &mut Self {
        self.add_argument(short, long, "", desc, HasArg::NO, Occur::Opt)
    }

    pub fn parse_arguments<T: IntoIterator>(&self, args: T) -> Res
        where T::Item: AsRef<OsStr>{

        let mut vals: Vec<Arg> = self.args.iter().map(|x| x.transform()).collect::<Vec<Arg>>();

        let mut free = Vec::new();
        let args: Vec<String> = args.into_iter().map(|i| {
            i.as_ref().to_str().
            ok_or_else(|| {
                Fail::UnknownArgument(format!("{:?}", i.as_ref()))
            }).map(|s| s.to_owned())
        }).collect::<::std::result::Result<Vec<_>, _>>()?;
        let mut args = args.into_iter().peekable();
        while let Some(cur) = args.next() {
            let mut names: Vec<Name> = Vec::new();
            let mut i_arg: Option<String> = None;
            if !OptParser::is_arg(&cur) {
                match self.style {
                    ParseStyle::FreeStyle => { free.push(cur); }
                    ParseStyle::StrictStyle => { return Err(Fail::UnexpectedArgument(cur.to_string())); }
                }
            } else if cur == "--" {
                free.extend(args);
                break
            } else {
                if cur.as_bytes()[1] == b'-' {
                    let tail: &str = &cur[2..];
                    let mut parts = tail.splitn(2, "=");
                    names.push(Name::from_str(parts.next().unwrap()));
                    if let Some(rest) = parts.next() {
                        i_arg = Some(rest.to_string());
                    }
                } else {
                    for (j, ch) in cur.char_indices().skip(1) {
                        names.push(Name::Short(ch));
                    }
                }
            }

            for name in names.iter() {
                let opt_id = match self.find_opt(name) {
                    Some(id) => id,
                    None => { return Err(Fail::UnknownArgument(name.to_string())) },
                };
                let v: &mut Arg = &mut vals[opt_id];

                match v.hasarg {
                    HasArg::NO => {
                        if i_arg.is_some() {
                            return Err(Fail::UnexpectedArgument(name.to_string()));
                        }
                    }
                    HasArg::May => {
                        if let Some(i_arg) = i_arg.take() {
                            v.value.push(ArgVal::Val(i_arg));
                        } else if args.peek().map_or(true, |n| OptParser::is_arg(n)) {
                            v.value.push(ArgVal::Given);
                        } else {
                            v.value.push(match args.next() {
                                Some(i_arg) => ArgVal::Val(i_arg),
                                None => ArgVal::Given
                            });
                        }
                    }
                    HasArg::YES => {
                        if i_arg.is_some() {
                            v.value.push(ArgVal::Val(i_arg.clone().unwrap()));
                        } else if let Some(i_arg) = args.next() {
                            v.value.push(ArgVal::Val(i_arg));
                        } else {
                            return Err(Fail::UnexpectedArgument(name.to_string()));
                        }
                    }
                }

                v.occured += 1;
            }
        }

        let table = OptTable {
            opts: vals,
            free: Some(free),
            parser: &self,
        };

        return table.check_occur();
    }

    pub fn usage(&self, brief: &str) -> String {
        self.usage_with_format(|opts|
            format!("{}\n\nOptions:\n{}\n", brief, opts.collect::<Vec<String>>().join("\n")))
    }

    fn usage_with_format(&self, mut formatter: impl FnMut(&mut Iterator<Item=String>) -> String) -> String {
        formatter(&mut self.usage_items())
    }

    fn usage_items<'a>(&'a self) -> Box<Iterator<Item=String> + 'a> {

        let rows = self.args.iter().map(move |optref| {
           let Argument {
                short_name,
                long_name,
                hint,
                desc,
                hasarg,
                .. } = (*optref).clone();

            let mut row: String = "    ".to_string();
            match short_name {
                Some(nm) => {
                    row.push('-');
                    row.push_str(nm.to_string().as_ref());
                    row.push_str("  ");
                }
                None => { println!("short name empty.") }
            }

            match long_name {
                Some(nm) => {
                    row.push_str("--");
                    row.push_str(nm.to_string().as_ref());
                    row.push_str("  ");
                }
                None => { println!("long name empty.") }
            }

            match hasarg {
                HasArg::NO => {}
                HasArg::YES => {
                    row.push_str(&hint);
                }
                HasArg::May => {
                    row.push('[');
                    row.push_str(&hint);
                    row.push(']');
                }
            }

            if row.len() < 20 {
                for _ in 0..(24 - row.len()) {
                    row.push(' ');
                }
            } else {
                row.push('\n');
                for _ in 0..6 {
                    row.push_str("    ");
                }
            }
            row.push_str(&desc);

            row
        });

        Box::new(rows)
    }
}

#[derive(Clone)]
pub enum ArgVal {
    Val(String),
    Given,
}

#[derive(Clone)]
struct Arg<'a> {
    value: Vec<ArgVal>,
    hasarg: HasArg,
    occur: Occur,
    occured: usize,
    submem: Vec<Arg<'a>>,
    argrc: &'a Argument,
}

pub struct OptTable<'a> {
    opts: Vec<Arg<'a>>,
    pub free: Option<Vec<String>>,
    parser: &'a OptParser,
}

impl<'a> OptTable<'a> {
    fn check_occur(self) -> Res<'a> {
        for opt in self.opts.iter() {
            if opt.occur == Occur::Once {
                match opt.occured {
                    0 => {
                        let mut err_info = String::new();
                        if let Some(ref long) = opt.argrc.long_name {
                            err_info.push_str(" --");
                            err_info.push_str(long.to_string().as_ref());
                            err_info.push(' ');
                        }
                        if let Some(ref short) = opt.argrc.short_name {
                            err_info.push_str(" -");
                            err_info.push_str(short.to_string().as_ref());
                            err_info.push(' ');
                        }
                        return Err(Fail::MissingArgument(err_info));
                    }
                    1 => { }
                    _ => {
                        let mut err_info = String::new();
                        if let Some(ref long) = opt.argrc.long_name {
                            err_info.push_str(" --");
                            err_info.push_str(long.to_string().as_ref());
                            err_info.push(' ');
                        }
                        if let Some(ref short) = opt.argrc.short_name {
                            err_info.push_str(" -");
                            err_info.push_str(short.to_string().as_ref());
                            err_info.push(' ');
                        }
                        return Err(Fail::DuplicatedArgument(err_info));
                    }
                }
            }
            if opt.occur == Occur::Opt {
                match opt.occured {
                    0 | 1 => { }
                    _ => {
                        let mut err_info = String::new();
                        if let Some(ref long) = opt.argrc.long_name {
                            err_info.push_str(" --");
                            err_info.push_str(long.to_string().as_ref());
                            err_info.push(' ');
                        }
                        if let Some(ref short) = opt.argrc.short_name {
                            err_info.push_str(" -");
                            err_info.push_str(short.to_string().as_ref());
                            err_info.push(' ');
                        }
                        return Err(Fail::DuplicatedArgument(err_info));
                    }
                }
            }
        }
        return Ok(self);
    }

    pub fn get_vals(&self, name: &str) -> Vec<ArgVal> {
        match self.parser.find_opt(&Name::from_str(name)) {
            Some(id) => self.opts[id].value.clone(),
            None => panic!("No option '{}' defined.", name)
        }
    }

    pub fn get_val(&self, name: &str) -> Option<String> {
        let id = match self.parser.find_opt(&Name::from_str(name)) {
            Some(id) => id,
            None => panic!("No option '{}' defined.", name),
        };

        match self.opts[id].value[0] {
            ArgVal::Val(ref s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn get_flag(&self, name: &str) -> bool {
        match self.parser.find_opt(&Name::from_str(name)) {
            Some(id) => match self.opts[id].occured {0 => false, _ => true},
            None => panic!("No option '{}' defined.", name)
        }
    }
}

