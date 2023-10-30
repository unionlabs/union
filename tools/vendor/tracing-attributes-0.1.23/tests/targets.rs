use tracing::subscriber::with_default;
use tracing_attributes::instrument;
use tracing_mock::*;

#[instrument]
fn default_target() {}

#[instrument(target = "my_target")]
fn custom_target() {}

mod my_mod {
    use tracing_attributes::instrument;

    pub const MODULE_PATH: &str = module_path!();

    #[instrument]
    pub fn default_target() {}

    #[instrument(target = "my_other_target")]
    pub fn custom_target() {}
}

#[test]
fn default_targets() {
    let (subscriber, handle) = subscriber::mock()
        .new_span(
            span::mock()
                .named("default_target")
                .with_target(module_path!()),
        )
        .enter(
            span::mock()
                .named("default_target")
                .with_target(module_path!()),
        )
        .exit(
            span::mock()
                .named("default_target")
                .with_target(module_path!()),
        )
        .new_span(
            span::mock()
                .named("default_target")
                .with_target(my_mod::MODULE_PATH),
        )
        .enter(
            span::mock()
                .named("default_target")
                .with_target(my_mod::MODULE_PATH),
        )
        .exit(
            span::mock()
                .named("default_target")
                .with_target(my_mod::MODULE_PATH),
        )
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        default_target();
        my_mod::default_target();
    });

    handle.assert_finished();
}

#[test]
fn custom_targets() {
    let (subscriber, handle) = subscriber::mock()
        .new_span(span::mock().named("custom_target").with_target("my_target"))
        .enter(span::mock().named("custom_target").with_target("my_target"))
        .exit(span::mock().named("custom_target").with_target("my_target"))
        .new_span(
            span::mock()
                .named("custom_target")
                .with_target("my_other_target"),
        )
        .enter(
            span::mock()
                .named("custom_target")
                .with_target("my_other_target"),
        )
        .exit(
            span::mock()
                .named("custom_target")
                .with_target("my_other_target"),
        )
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        custom_target();
        my_mod::custom_target();
    });

    handle.assert_finished();
}
