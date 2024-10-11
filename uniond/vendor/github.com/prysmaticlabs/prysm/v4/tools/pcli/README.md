## Pcli (Prysm CLI)

This is a utility to help users perform Ethereum consensus specific commands.

### Usage

*Name:*  
   **pcli** - A command line utility to run Ethereum consensus specific commands

*Usage:*  
   pcli [global options] command [command options] [arguments...]

*Commands:*
     help, h  Shows a list of commands or help for one command
   state-transition:
     state-transition  Subcommand to run manual state transitions


*Flags:*  
   --help, -h     show help (default: false)
   --version, -v  print the version (default: false)

*State Transition Subcommand:*
   pcli state-transition - Subcommand to run manual state transitions

*State Transition Usage:*:
   pcli state-transition [command options] [arguments...]


*State Transition Flags:*
   --block-path value              Path to block file(ssz)
   --pre-state-patch value           Path to pre state file(ssz)
   --expected-post-state-path value  Path to expected post state file(ssz)
   --help, -h                     show help (default: false)



### Example

To use pcli manual state transition:

```
bazel run //tools/pcli:pcli -- state-transition --block-path /path/to/block.ssz --pre-state-path /path/to/state.ssz
```

