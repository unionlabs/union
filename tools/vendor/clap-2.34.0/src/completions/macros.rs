macro_rules! w {
    ($buf:expr, $to_w:expr) => {
        match $buf.write_all($to_w) {
            Ok(..) => (),
            Err(..) => panic!("Failed to write to completions file"),
        }
    };
}

macro_rules! get_zsh_arg_conflicts {
    ($p:ident, $arg:ident, $msg:ident) => {
        if let Some(conf_vec) = $arg.blacklist() {
            let mut v = vec![];
            for arg_name in conf_vec {
                let arg = $p.find_any_arg(arg_name).expect($msg);
                if let Some(s) = arg.short() {
                    v.push(format!("-{}", s));
                }
                if let Some(l) = arg.long() {
                    v.push(format!("--{}", l));
                }
            }
            v.join(" ")
        } else {
            String::new()
        }
    };
}
