/*
mkwinsyscall generates windows system call bodies

It parses all files specified on command line containing function
prototypes (like syscall_windows.go) and prints system call bodies
to standard output.

The prototypes are marked by lines beginning with "//sys" and read
like func declarations if //sys is replaced by func, but:

  - The parameter lists must give a name for each argument. This
    includes return parameters.

  - The parameter lists must give a type for each argument:
    the (x, y, z int) shorthand is not allowed.

  - If the return parameter is an error number, it must be named err.

  - If go func name needs to be different from its winapi dll name,
    the winapi name could be specified at the end, after "=" sign, like

    //sys LoadLibrary(libname string) (handle uint32, err error) = LoadLibraryA

  - Each function that returns err needs to supply a condition, that
    return value of winapi will be tested against to detect failure.
    This would set err to windows "last-error", otherwise it will be nil.
    The value can be provided at end of //sys declaration, like

    //sys LoadLibrary(libname string) (handle uint32, err error) [failretval==-1] = LoadLibraryA

    and is [failretval==0] by default.

  - If the function name ends in a "?", then the function not existing is non-
    fatal, and an error will be returned instead of panicking.

Usage:

	mkwinsyscall [flags] [path ...]

Flags

	-output string
	      Output file name (standard output if omitted).
	-sort
	      Sort DLL and function declarations (default true).
	      Intended to help transition from  older versions of mkwinsyscall by making diffs
	      easier to read and understand.
	-systemdll
	      Whether all DLLs should be loaded from the Windows system directory (default true).
	-trace
	      Generate print statement after every syscall.
	-utf16
	      Encode string arguments as UTF-16 for syscalls not ending in 'A' or 'W' (default true).
	-winio
	      Import this package ("github.com/Microsoft/go-winio").
*/
package main
