use core::fmt;
use std::fmt::Debug;

use tracing::field::Visit;
use tracing_subscriber::{
    field::{MakeVisitor, VisitFmt, VisitOutput},
    fmt::format::{DefaultVisitor, Writer},
};
use valuable::{NamedValues, Slice, Valuable, Value};

pub struct CustomVisitor<'a> {
    inner: DefaultVisitor<'a>,
    res: fmt::Result,
}

impl<'a> Visit for CustomVisitor<'a> {
    fn record_value(&mut self, field: &tracing::field::Field, value: Value<'_>) {
        let value_pretty_debug = ValuePrettyDebug { value, res: Ok(()) };
        self.inner.record_debug(field, &value_pretty_debug);
        self.res = value_pretty_debug.res;
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.inner.record_debug(field, value);
    }
}

impl<'a> VisitFmt for CustomVisitor<'a> {
    fn writer(&mut self) -> &mut dyn fmt::Write {
        self.inner.writer()
    }
}

impl<'a> VisitOutput<fmt::Result> for CustomVisitor<'a> {
    fn finish(self) -> fmt::Result {
        self.res
    }
}

struct ValuePrettyDebug<'a> {
    value: Value<'a>,
    res: fmt::Result,
}

impl<'a> Debug for ValuePrettyDebug<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.visit(&mut PrettyVisitor {
            f,
            prefix_with_comma: false,
            res: Ok(()),
        });

        Ok(())
    }
}

struct PrettyVisitor<'a, 'b> {
    f: &'a mut std::fmt::Formatter<'b>,
    prefix_with_comma: bool,
    res: fmt::Result,
}

impl<'a, 'b> valuable::Visit for PrettyVisitor<'a, 'b> {
    fn visit_value(&mut self, value: Value<'_>) {
        if self.prefix_with_comma {
            self.res = write!(self.f, ",");
        }

        match value {
            Value::Bool(v) => {
                self.res = write!(self.f, "{v}");
            }
            Value::Char(v) => {
                self.res = write!(self.f, "{v}");
            }
            Value::F32(v) => {
                self.res = write!(self.f, "{v}");
            }
            Value::F64(v) => {
                self.res = write!(self.f, "{v}");
            }
            Value::I8(v) => {
                self.res = write!(self.f, "{v}");
            }
            Value::I16(v) => {
                self.res = write!(self.f, "{v}");
            }
            Value::I32(v) => {
                self.res = write!(self.f, "{v}");
            }
            Value::I64(v) => {
                self.res = write!(self.f, "{v}");
            }
            Value::I128(v) => {
                self.res = write!(self.f, "{v}");
            }
            Value::Isize(v) => {
                self.res = write!(self.f, "{v}");
            }
            Value::String(v) => {
                self.res = write!(self.f, "{v:?}");
            }
            Value::U8(v) => {
                self.res = write!(self.f, "{v}");
            }
            Value::U16(v) => {
                self.res = write!(self.f, "{v}");
            }
            Value::U32(v) => {
                self.res = write!(self.f, "{v}");
            }
            Value::U64(v) => {
                self.res = write!(self.f, "{v}");
            }
            Value::U128(v) => {
                self.res = write!(self.f, "{v}");
            }
            Value::Usize(v) => {
                self.res = write!(self.f, "{v}");
            }
            Value::Path(v) => {
                self.res = write!(self.f, "{v:?}");
            }
            Value::Error(v) => {
                self.res = write!(self.f, "{v}");
            }
            Value::Listable(v) => {
                self.res = write!(self.f, "[");

                v.visit(self);

                self.res = write!(self.f, "]");
            }
            Value::Mappable(v) => v.visit(self),
            Value::Structable(v) => v.visit(self),
            Value::Enumerable(v) => {
                let variant = v.variant();
                let variant_name = variant.name();

                self.res = write!(self.f, "{variant_name}:");

                v.visit(self)
            }
            Value::Tuplable(v) => v.visit(self),
            Value::Unit => todo!(),
            _ => todo!(),
        }
    }

    fn visit_named_fields(&mut self, named_values: &NamedValues<'_>) {
        let len = named_values.len();

        self.res = write!(self.f, "{{");
        for (i, (k, v)) in named_values.into_iter().enumerate() {
            self.res = write!(self.f, "{}=", k.name());
            v.visit(self);
            if i < len - 1 {
                self.res = write!(self.f, ",");
            }
        }
        self.res = write!(self.f, "}}");
    }

    fn visit_unnamed_fields(&mut self, values: &[Value<'_>]) {
        let len = values.len();

        if len > 1 {
            self.res = write!(self.f, "(");
        }
        for (i, v) in values.iter().enumerate() {
            v.visit(self);
            if i < len - 1 {
                self.res = write!(self.f, ",");
            }
        }
        if len > 1 {
            self.res = write!(self.f, ")");
        }
    }

    fn visit_primitive_slice(&mut self, slice: Slice<'_>) {
        let len = slice.len();

        self.res = write!(self.f, "[");
        for (i, v) in slice.into_iter().enumerate() {
            v.visit(self);
            if i < len - 1 {
                self.res = write!(self.f, ",");
            }
        }
        self.res = write!(self.f, "]");
    }

    fn visit_entry(&mut self, key: Value<'_>, value: Value<'_>) {
        key.visit(self);
        self.res = write!(self.f, "=");
        value.visit(self);
    }
}

pub struct MakeCustomVisitor {}

impl<'a> MakeVisitor<Writer<'a>> for MakeCustomVisitor {
    type Visitor = CustomVisitor<'a>;

    fn make_visitor(&self, target: Writer<'a>) -> Self::Visitor {
        CustomVisitor {
            inner: DefaultVisitor::new(target, true),
            res: Ok(()),
        }
    }
}
