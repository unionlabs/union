fn main() {
    let mut build = cc::Build::new();
    build.file("vendor/wepoll/wepoll.c");

    if cfg!(feature = "null-overlapped-wakeups-patch") {
        build.define("NULL_OVERLAPPED_WAKEUPS_PATCH", None);
    }

    build.compile("wepoll");

    // We regenerate binding code and check it in. (See generate_bindings.bat)
}
