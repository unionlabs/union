#[cfg(feature = "yaml")]
macro_rules! yaml_tuple2 {
    ($a:ident, $v:ident, $c:ident) => {{
        if let Some(vec) = $v.as_vec() {
            for ys in vec {
                if let Some(tup) = ys.as_vec() {
                    debug_assert_eq!(2, tup.len());
                    $a = $a.$c(yaml_str!(tup[0]), yaml_str!(tup[1]));
                } else {
                    panic!("Failed to convert YAML value to vec");
                }
            }
        } else {
            panic!("Failed to convert YAML value to vec");
        }
        $a
    }};
}

#[cfg(feature = "yaml")]
macro_rules! yaml_tuple3 {
    ($a:ident, $v:ident, $c:ident) => {{
        if let Some(vec) = $v.as_vec() {
            for ys in vec {
                if let Some(tup) = ys.as_vec() {
                    debug_assert_eq!(3, tup.len());
                    $a = $a.$c(yaml_str!(tup[0]), yaml_opt_str!(tup[1]), yaml_str!(tup[2]));
                } else {
                    panic!("Failed to convert YAML value to vec");
                }
            }
        } else {
            panic!("Failed to convert YAML value to vec");
        }
        $a
    }};
}

#[cfg(feature = "yaml")]
macro_rules! yaml_vec_or_str {
    ($v:ident, $a:ident, $c:ident) => {{
        let maybe_vec = $v.as_vec();
        if let Some(vec) = maybe_vec {
            for ys in vec {
                if let Some(s) = ys.as_str() {
                    $a = $a.$c(s);
                } else {
                    panic!("Failed to convert YAML value {:?} to a string", ys);
                }
            }
        } else {
            if let Some(s) = $v.as_str() {
                $a = $a.$c(s);
            } else {
                panic!(
                    "Failed to convert YAML value {:?} to either a vec or string",
                    $v
                );
            }
        }
        $a
    }};
}

#[cfg(feature = "yaml")]
macro_rules! yaml_opt_str {
    ($v:expr) => {{
        if $v.is_null() {
            Some(
                $v.as_str()
                    .unwrap_or_else(|| panic!("failed to convert YAML {:?} value to a string", $v)),
            )
        } else {
            None
        }
    }};
}

#[cfg(feature = "yaml")]
macro_rules! yaml_str {
    ($v:expr) => {{
        $v.as_str()
            .unwrap_or_else(|| panic!("failed to convert YAML {:?} value to a string", $v))
    }};
}

#[cfg(feature = "yaml")]
macro_rules! yaml_to_str {
    ($a:ident, $v:ident, $c:ident) => {{
        $a.$c(yaml_str!($v))
    }};
}

#[cfg(feature = "yaml")]
macro_rules! yaml_to_bool {
    ($a:ident, $v:ident, $c:ident) => {{
        $a.$c($v
            .as_bool()
            .unwrap_or_else(|| panic!("failed to convert YAML {:?} value to a string", $v)))
    }};
}

#[cfg(feature = "yaml")]
macro_rules! yaml_to_u64 {
    ($a:ident, $v:ident, $c:ident) => {{
        $a.$c($v
            .as_i64()
            .unwrap_or_else(|| panic!("failed to convert YAML {:?} value to a string", $v))
            as u64)
    }};
}

#[cfg(feature = "yaml")]
macro_rules! yaml_to_usize {
    ($a:ident, $v:ident, $c:ident) => {{
        $a.$c($v
            .as_i64()
            .unwrap_or_else(|| panic!("failed to convert YAML {:?} value to a string", $v))
            as usize)
    }};
}
