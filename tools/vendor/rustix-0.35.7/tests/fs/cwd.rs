/// Make sure we can use `cwd` in const contexts.
#[allow(dead_code)]
const CWD: rustix::fd::BorrowedFd<'static> = rustix::fs::cwd();
