use debugger_test::debugger_test;
use url::Url;

#[inline(never)]
fn __break() {}

#[debugger_test(
    debugger = "cdb",
    commands = "
    .nvlist

    dx base_url

    dx url_with_non_special_scheme

    dx url_with_user_pass_port_query_fragments

    dx url_blob

    dx url_with_base

    dx url_with_base_replaced

    dx url_with_comma",
    expected_statements = r#"
    pattern:debugger_visualizer-.*\.exe \(embedded NatVis ".*-[0-9]+\.natvis"\)

    base_url         : "http://example.org/foo/bar" [Type: url::Url]
    [<Raw View>]     [Type: url::Url]
    [scheme]         : "http"
    [host]           : "example.org"
    [path]           : "/foo/bar"

    url_with_non_special_scheme : "non-special://test/x" [Type: url::Url]
    [<Raw View>]     [Type: url::Url]
    [scheme]         : "non-special"
    [host]           : "test"
    [path]           : "/x"

    url_with_user_pass_port_query_fragments : "http://user:pass@foo:21/bar;par?b#c" [Type: url::Url]
    [<Raw View>]     [Type: url::Url]
    [scheme]         : "http"
    [username]       : "user"
    [host]           : "foo"
    [port]           : 21
    [path]           : "/bar;par"
    [query]          : "b"
    [fragment]       : "c"

    url_blob         : "blob:https://example.com:443/" [Type: url::Url]
    [<Raw View>]     [Type: url::Url]
    [scheme]         : "blob"
    [path]           : "https://example.com:443/"

    url_with_base    : "http://example.org/a%2fc" [Type: url::Url]
    [<Raw View>]     [Type: url::Url]
    [scheme]         : "http"
    [host]           : "example.org"
    [path]           : "/a%2fc"

    url_with_base_replaced : "http://[::7f00:1]/" [Type: url::Url]
    [<Raw View>]     [Type: url::Url]
    [scheme]         : "http"
    [host]           : "[::7f00:1]"
    [path]           : "/"

    url_with_comma   : "data:text/html,test#test" [Type: url::Url]
    [<Raw View>]     [Type: url::Url]
    [scheme]         : "data"
    [path]           : "text/html,test"
    [fragment]       : "test"
    "#
)]
fn test_url_visualizer() {
    // Copied from https://github.com/web-platform-tests/wpt/blob/master/url/
    let base_url = Url::parse("http://example.org/foo/bar").unwrap();
    assert_eq!(base_url.as_str(), "http://example.org/foo/bar");

    let url_with_non_special_scheme = Url::parse("non-special://:@test/x").unwrap();
    assert_eq!(url_with_non_special_scheme.as_str(), "non-special://test/x");

    let url_with_user_pass_port_query_fragments =
        Url::parse("http://user:pass@foo:21/bar;par?b#c").unwrap();
    assert_eq!(
        url_with_user_pass_port_query_fragments.as_str(),
        "http://user:pass@foo:21/bar;par?b#c"
    );

    let url_blob = Url::parse("blob:https://example.com:443/").unwrap();
    assert_eq!(url_blob.as_str(), "blob:https://example.com:443/");

    let url_with_base = base_url.join("/a%2fc").unwrap();
    assert_eq!(url_with_base.as_str(), "http://example.org/a%2fc");

    let url_with_base_replaced = base_url.join("http://[::127.0.0.1]").unwrap();
    assert_eq!(url_with_base_replaced.as_str(), "http://[::7f00:1]/");

    let url_with_comma = base_url.join("data:text/html,test#test").unwrap();
    assert_eq!(url_with_comma.as_str(), "data:text/html,test#test");

    __break();
}
