*Jump to [source](git.rs)*

Git is an example of several common subcommand patterns.

Help:
```console
$ git
? failed
git 
A fictional versioning CLI

USAGE:
    git[EXE] <SUBCOMMAND>

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    add      adds things
    clone    Clones repos
    help     Print this message or the help of the given subcommand(s)
    push     pushes things
    stash    

$ git help
git 
A fictional versioning CLI

USAGE:
    git[EXE] <SUBCOMMAND>

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    add      adds things
    clone    Clones repos
    help     Print this message or the help of the given subcommand(s)
    push     pushes things
    stash    

$ git help add
git[EXE]-add 
adds things

USAGE:
    git[EXE] add <PATH>...

ARGS:
    <PATH>...    Stuff to add

OPTIONS:
    -h, --help    Print help information

```

A basic argument:
```console
$ git add
? failed
git[EXE]-add 
adds things

USAGE:
    git[EXE] add <PATH>...

ARGS:
    <PATH>...    Stuff to add

OPTIONS:
    -h, --help    Print help information

$ git add Cargo.toml Cargo.lock
Adding ["Cargo.toml", "Cargo.lock"]

```

Default subcommand:
```console
$ git stash -h
git[EXE]-stash 

USAGE:
    git[EXE] stash [OPTIONS]
    git[EXE] stash <SUBCOMMAND>

OPTIONS:
    -h, --help                 Print help information
    -m, --message <MESSAGE>    

SUBCOMMANDS:
    apply    
    help     Print this message or the help of the given subcommand(s)
    pop      
    push     

$ git stash push -h
git[EXE]-stash-push 

USAGE:
    git[EXE] stash push [OPTIONS]

OPTIONS:
    -h, --help                 Print help information
    -m, --message <MESSAGE>    

$ git stash pop -h
git[EXE]-stash-pop 

USAGE:
    git[EXE] stash pop [STASH]

ARGS:
    <STASH>    

OPTIONS:
    -h, --help    Print help information

$ git stash -m "Prototype"
Pushing Some("Prototype")

$ git stash pop
Popping None

$ git stash push -m "Prototype"
Pushing Some("Prototype")

$ git stash pop
Popping None

```

External subcommands:
```console
$ git custom-tool arg1 --foo bar
Calling out to "custom-tool" with ["arg1", "--foo", "bar"]

```
