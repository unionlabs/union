use std::fmt::{self, Display};

#[derive(Copy, Clone)]
pub struct Error {
    pub msg: &'static str,
    pub label: Option<&'static str>,
    pub note: Option<&'static str>,
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.msg.fmt(formatter)
    }
}

pub static ERRORS: &[Error] = &[
    BOX_CXX_TYPE,
    CXXBRIDGE_RESERVED,
    CXX_STRING_BY_VALUE,
    CXX_TYPE_BY_VALUE,
    DISCRIMINANT_OVERFLOW,
    DOT_INCLUDE,
    DOUBLE_UNDERSCORE,
    RESERVED_LIFETIME,
    RUST_TYPE_BY_VALUE,
    UNSUPPORTED_TYPE,
    USE_NOT_ALLOWED,
];

pub static BOX_CXX_TYPE: Error = Error {
    msg: "Box of a C++ type is not supported yet",
    label: None,
    note: Some("hint: use UniquePtr<> or SharedPtr<>"),
};

pub static CXXBRIDGE_RESERVED: Error = Error {
    msg: "identifiers starting with cxxbridge are reserved",
    label: Some("reserved identifier"),
    note: Some("identifiers starting with cxxbridge are reserved"),
};

pub static CXX_STRING_BY_VALUE: Error = Error {
    msg: "C++ string by value is not supported",
    label: None,
    note: Some("hint: wrap it in a UniquePtr<>"),
};

pub static CXX_TYPE_BY_VALUE: Error = Error {
    msg: "C++ type by value is not supported",
    label: None,
    note: Some("hint: wrap it in a UniquePtr<> or SharedPtr<>"),
};

pub static DISCRIMINANT_OVERFLOW: Error = Error {
    msg: "discriminant overflow on value after ",
    label: Some("discriminant overflow"),
    note: Some("note: explicitly set `= 0` if that is desired outcome"),
};

pub static DOT_INCLUDE: Error = Error {
    msg: "#include relative to `.` or `..` is not supported in Cargo builds",
    label: Some("#include relative to `.` or `..` is not supported in Cargo builds"),
    note: Some("note: use a path starting with the crate name"),
};

pub static DOUBLE_UNDERSCORE: Error = Error {
    msg: "identifiers containing double underscore are reserved in C++",
    label: Some("reserved identifier"),
    note: Some("identifiers containing double underscore are reserved in C++"),
};

pub static RESERVED_LIFETIME: Error = Error {
    msg: "invalid lifetime parameter name: `'static`",
    label: Some("'static is a reserved lifetime name"),
    note: None,
};

pub static RUST_TYPE_BY_VALUE: Error = Error {
    msg: "opaque Rust type by value is not supported",
    label: None,
    note: Some("hint: wrap it in a Box<>"),
};

pub static UNSUPPORTED_TYPE: Error = Error {
    msg: "unsupported type: ",
    label: Some("unsupported type"),
    note: None,
};

pub static USE_NOT_ALLOWED: Error = Error {
    msg: "`use` items are not allowed within cxx bridge",
    label: Some("not allowed"),
    note: Some(
        "`use` items are not allowed within cxx bridge; only types defined\n\
         within your bridge, primitive types, or types exported by the cxx\n\
         crate may be used",
    ),
};
