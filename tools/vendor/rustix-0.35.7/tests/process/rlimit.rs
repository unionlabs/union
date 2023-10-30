use rustix::process::{Resource, Rlimit};

#[test]
fn test_getrlimit() {
    let lim = rustix::process::getrlimit(Resource::Stack);
    assert_ne!(lim.current, Some(0));
    assert_ne!(lim.maximum, Some(0));
}

#[test]
fn test_setrlimit() {
    let old = rustix::process::getrlimit(Resource::Core);
    let new = Rlimit {
        current: Some(0),
        maximum: Some(4096),
    };
    assert_ne!(old, new);
    rustix::process::setrlimit(Resource::Core, new.clone()).unwrap();

    let lim = rustix::process::getrlimit(Resource::Core);
    assert_eq!(lim, new);

    #[cfg(any(target_os = "android", target_os = "linux"))]
    {
        let new = Rlimit {
            current: Some(0),
            maximum: Some(0),
        };

        let first = rustix::process::getrlimit(Resource::Core);

        let old = match rustix::process::prlimit(None, Resource::Core, new.clone()) {
            Ok(rlimit) => rlimit,
            Err(rustix::io::Errno::NOSYS) => return,
            Err(e) => Err(e).unwrap(),
        };

        assert_eq!(first, old);

        let other = Rlimit {
            current: Some(0),
            maximum: Some(0),
        };

        let again =
            rustix::process::prlimit(Some(rustix::process::getpid()), Resource::Core, other)
                .unwrap();

        assert_eq!(again, new);
    }
}
