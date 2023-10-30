*Jump to [source](pacman.rs)*

[`pacman`](https://wiki.archlinux.org/index.php/pacman) defines subcommands via flags.

Here, `-S` is a short flag subcommand:
```console
$ pacman -S package
Installing package...

```

Here `--sync` is a long flag subcommand:
```console
$ pacman --sync package
Installing package...

```

Now the short flag subcommand (`-S`) with a long flag:
```console
$ pacman -S --search name
Searching for name...

```

And the various forms of short flags that work:
```console
$ pacman -S -s name
Searching for name...

$ pacman -Ss name
Searching for name...

```
*(users can "stack" short subcommands with short flags or with other short flag subcommands)*

In the help, this looks like:
```console
$ pacman -h
pacman 5.2.1
Pacman Development Team
package manager utility

USAGE:
    pacman[EXE] <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    help                Print this message or the help of the given subcommand(s)
    query -Q --query    Query the package database.
    sync -S --sync      Synchronize packages.

$ pacman -S -h
pacman[EXE]-sync 
Synchronize packages.

USAGE:
    pacman[EXE] {sync|--sync|-S} [OPTIONS] [--] [package]...

ARGS:
    <package>...    packages

OPTIONS:
    -h, --help                  Print help information
    -i, --info                  view package information
    -s, --search <search>...    search remote repositories for matching strings

```

And errors:
```console
$ pacman -S -s foo -i bar
? failed
error: The argument '--search <search>...' cannot be used with '--info'

USAGE:
    pacman[EXE] {sync|--sync|-S} --search <search>... <package>...

For more information try --help

```

**NOTE:** Keep in mind that subcommands, flags, and long flags are *case sensitive*: `-Q` and `-q` are different flags/subcommands. For example, you can have both `-Q` subcommand and `-q` flag, and they will be properly disambiguated.
Let's make a quick program to illustrate.
