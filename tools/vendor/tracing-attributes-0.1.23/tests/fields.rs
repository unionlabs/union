use tracing::subscriber::with_default;
use tracing_attributes::instrument;
use tracing_mock::field::mock;
use tracing_mock::span::NewSpan;
use tracing_mock::*;

#[instrument(fields(foo = "bar", dsa = true, num = 1))]
fn fn_no_param() {}

#[instrument(fields(foo = "bar"))]
fn fn_param(param: u32) {}

#[instrument(fields(foo = "bar", empty))]
fn fn_empty_field() {}

#[instrument(fields(len = s.len()))]
fn fn_expr_field(s: &str) {}

#[instrument(fields(s.len = s.len(), s.is_empty = s.is_empty()))]
fn fn_two_expr_fields(s: &str) {
    let _ = s;
}

#[instrument(fields(%s, s.len = s.len()))]
fn fn_clashy_expr_field(s: &str) {
    let _ = s;
}

#[instrument(fields(s = "s"))]
fn fn_clashy_expr_field2(s: &str) {
    let _ = s;
}

#[instrument(fields(s = &s))]
fn fn_string(s: String) {
    let _ = s;
}

#[derive(Debug)]
struct HasField {
    my_field: &'static str,
}

impl HasField {
    #[instrument(fields(my_field = self.my_field), skip(self))]
    fn self_expr_field(&self) {}
}

#[test]
fn fields() {
    let span = span::mock().with_field(
        mock("foo")
            .with_value(&"bar")
            .and(mock("dsa").with_value(&true))
            .and(mock("num").with_value(&1))
            .only(),
    );
    run_test(span, || {
        fn_no_param();
    });
}

#[test]
fn expr_field() {
    let span = span::mock().with_field(
        mock("s")
            .with_value(&"hello world")
            .and(mock("len").with_value(&"hello world".len()))
            .only(),
    );
    run_test(span, || {
        fn_expr_field("hello world");
    });
}

#[test]
fn two_expr_fields() {
    let span = span::mock().with_field(
        mock("s")
            .with_value(&"hello world")
            .and(mock("s.len").with_value(&"hello world".len()))
            .and(mock("s.is_empty").with_value(&false))
            .only(),
    );
    run_test(span, || {
        fn_two_expr_fields("hello world");
    });
}

#[test]
fn clashy_expr_field() {
    let span = span::mock().with_field(
        // Overriding the `s` field should record `s` as a `Display` value,
        // rather than as a `Debug` value.
        mock("s")
            .with_value(&tracing::field::display("hello world"))
            .and(mock("s.len").with_value(&"hello world".len()))
            .only(),
    );
    run_test(span, || {
        fn_clashy_expr_field("hello world");
    });

    let span = span::mock().with_field(mock("s").with_value(&"s").only());
    run_test(span, || {
        fn_clashy_expr_field2("hello world");
    });
}

#[test]
fn self_expr_field() {
    let span = span::mock().with_field(mock("my_field").with_value(&"hello world").only());
    run_test(span, || {
        let has_field = HasField {
            my_field: "hello world",
        };
        has_field.self_expr_field();
    });
}

#[test]
fn parameters_with_fields() {
    let span = span::mock().with_field(
        mock("foo")
            .with_value(&"bar")
            .and(mock("param").with_value(&1u32))
            .only(),
    );
    run_test(span, || {
        fn_param(1);
    });
}

#[test]
fn empty_field() {
    let span = span::mock().with_field(mock("foo").with_value(&"bar").only());
    run_test(span, || {
        fn_empty_field();
    });
}

#[test]
fn string_field() {
    let span = span::mock().with_field(mock("s").with_value(&"hello world").only());
    run_test(span, || {
        fn_string(String::from("hello world"));
    });
}

fn run_test<F: FnOnce() -> T, T>(span: NewSpan, fun: F) {
    let (subscriber, handle) = subscriber::mock()
        .new_span(span)
        .enter(span::mock())
        .exit(span::mock())
        .done()
        .run_with_handle();

    with_default(subscriber, fun);
    handle.assert_finished();
}
