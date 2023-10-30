*Jump to [source](git-derive.rs)*

**This requires enabling the `derive` feature flag.**

Git is an example of several common subcommand patterns.

Help:
```console
$ git-derive
? failed
git 
A fictional versioning CLI

USAGE:
    git-derive[EXE] <SUBCOMMAND>

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    add      adds things
    clone    Clones repos
    help     Print this message or the help of the given subcommand(s)
    push     pushes things
    stash    

$ git-derive help
git 
A fictional versioning CLI

USAGE:
    git-derive[EXE] <SUBCOMMAND>

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    add      adds things
    clone    Clones repos
    help     Print this message or the help of the given subcommand(s)
    push     pushes things
    stash    

$ git-derive help add
git-derive[EXE]-add 
adds things

USAGE:
    git-derive[EXE] add <PATH>...

ARGS:
    <PATH>...    Stuff to add

OPTIONS:
    -h, --help    Print help information

```

A basic argument:
```console
$ git-derive add
? failed
git-derive[EXE]-add 
adds things

USAGE:
    git-derive[EXE] add <PATH>...

ARGS:
    <PATH>...    Stuff to add

OPTIONS:
    -h, --help    Print help information

$ git-derive add Cargo.toml Cargo.lock
Adding ["Cargo.toml", "Cargo.lock"]

```

Default subcommand:
```console
$ git-derive stash -h
git-derive[EXE]-stash 

USAGE:
    git-derive[EXE] stash [OPTIONS]
    git-derive[EXE] stash <SUBCOMMAND>

OPTIONS:
    -h, --help                 Print help information
    -m, --message <MESSAGE>    

SUBCOMMANDS:
    apply    
    help     Print this message or the help of the given subcommand(s)
    pop      
    push     

$ git-derive stash push -h
git-derive[EXE]-stash-push 

USAGE:
    git-derive[EXE] stash push [OPTIONS]

OPTIONS:
    -h, --help                 Print help information
    -m, --message <MESSAGE>    

$ git-derive stash pop -h
git-derive[EXE]-stash-pop 

USAGE:
    git-derive[EXE] stash pop [STASH]

ARGS:
    <STASH>    

OPTIONS:
    -h, --help    Print help information

$ git-derive stash -m "Prototype"
Pushing StashPush { message: Some("Prototype") }

$ git-derive stash pop
Popping None

$ git-derive stash push -m "Prototype"
Pushing StashPush { message: Some("Prototype") }

$ git-derive stash pop
Popping None

```

External subcommands:
```console
$ git-derive custom-tool arg1 --foo bar
Calling out to "custom-tool" with ["arg1", "--foo", "bar"]

```
