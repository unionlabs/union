use std::fmt::{self, Write};

use crate::{ValidationError, ValidationErrors, ValidationErrorsKind};

impl fmt::Display for ValidationError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(msg) = self.message.as_ref() {
            write!(fmt, "{}", msg)
        } else {
            write!(fmt, "Validation error: {} [{:?}]", self.code, self.params)
        }
    }
}

fn display_errors(
    fmt: &mut fmt::Formatter<'_>,
    errs: &ValidationErrorsKind,
    path: &str,
) -> fmt::Result {
    fn display_struct(
        fmt: &mut fmt::Formatter<'_>,
        errs: &ValidationErrors,
        path: &str,
    ) -> fmt::Result {
        let mut full_path = String::new();
        write!(&mut full_path, "{}.", path)?;
        let base_len = full_path.len();
        for (path, err) in errs.errors() {
            write!(&mut full_path, "{}", path)?;
            display_errors(fmt, err, &full_path)?;
            full_path.truncate(base_len);
        }
        Ok(())
    }

    match errs {
        ValidationErrorsKind::Field(errs) => {
            write!(fmt, "{}: ", path)?;
            let len = errs.len();
            for (idx, err) in errs.iter().enumerate() {
                if idx + 1 == len {
                    write!(fmt, "{}", err)?;
                } else {
                    write!(fmt, "{}, ", err)?;
                }
            }
            Ok(())
        }
        ValidationErrorsKind::Struct(errs) => display_struct(fmt, errs, path),
        ValidationErrorsKind::List(errs) => {
            let mut full_path = String::new();
            write!(&mut full_path, "{}", path)?;
            let base_len = full_path.len();
            for (idx, err) in errs.iter() {
                write!(&mut full_path, "[{}]", idx)?;
                display_struct(fmt, err, &full_path)?;
                full_path.truncate(base_len);
            }
            Ok(())
        }
    }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (idx, (path, err)) in self.errors().iter().enumerate() {
            display_errors(fmt, err, path)?;
            if idx + 1 < self.errors().len() {
                writeln!(fmt)?;
            }
        }
        Ok(())
    }
}
