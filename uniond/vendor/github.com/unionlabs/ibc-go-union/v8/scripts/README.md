Generally we should avoid shell scripting and write tests purely in Golang.
However, some libraries are not Goroutine-safe (e.g. app simulations cannot be run safely in parallel),
and OS-native threading may be more efficient for many parallel simulations, so we use shell scripts here.
