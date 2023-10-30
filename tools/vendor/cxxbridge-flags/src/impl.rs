#[allow(unused_assignments, unused_mut, unused_variables)]
pub const STD: &str = {
    let mut flags = ["-std=c++11", "/std:c++11"];

    #[cfg(feature = "c++14")]
    (flags = ["-std=c++14", "/std:c++14"]);

    #[cfg(feature = "c++17")]
    (flags = ["-std=c++17", "/std:c++17"]);

    #[cfg(feature = "c++20")]
    (flags = ["-std=c++20", "/std:c++20"]);

    let [mut flag, msvc_flag] = flags;

    #[cfg(target_env = "msvc")]
    (flag = msvc_flag);

    flag
};
