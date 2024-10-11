#!/usr/bin/python3
"""
The following script takes care of adding new/removing versions or 
replacing a version in the compatibility-test-matrices JSON files.

To use this script, you'll need to have Python 3.9+ installed.

Invocation:

By default, the script assumes that adding a new version is the desired operation.
Furthermore, it assumes that the compatibility-test-matrices directory is located
in the .github directory and the script is invoked from the root of the repository.

If any of the above is not true, you can use the '--type' and '--directory' flags
to specify the operation and the directory respectively.

Typically, an invocation would look like:

    scripts/update_compatibility_tests.py --recent_version v4.3.0 --new_version v4.4.0

The three operations currently added are:

 - ADD: Add a new version to the JSON files. Requires both '--recent_version' and
        '--new_version' options to be set.
 - REPLACE: Replace an existing version with a new one. Requires both '--recent_version'
            and '--new_version' options to be set.
 - REMOVE: Remove an existing version from the JSON files. Requires only the
           '--recent_version' options to be set.

For more information, use the '--help' flag to see the available options.    
"""
import argparse
import os
import json
import enum
from collections import defaultdict
from typing import Tuple, Generator, Optional, Dict, List, Any

# Directory to operate in
DIRECTORY: str = ".github/compatibility-test-matrices"
# JSON keys to search in.
KEYS: Tuple[str, str] = ("chain-a", "chain-b")
# Toggle if required. Indent = 2 matches our current formatting.
DUMP_ARGS: Dict[Any, Any] = {
    "indent": 2,
    "sort_keys": False,
    "ensure_ascii": False,
}
# Suggestions for recent and new versions.
SUGGESTION: str = "for example, v4.3.0 or 4.3.0"
# Supported Operations.
Operation = enum.Enum("Operation", ["ADD", "REMOVE", "REPLACE"])


def find_json_files(
    directory: str, ignores: Tuple[str] = (".",)
) -> Generator[str, None, None]:
    """Find JSON files in a directory. By default, ignore hidden directories."""
    for root, dirs, files in os.walk(directory):
        dirs[:] = (d for d in dirs if not d.startswith(ignores))
        for file_ in files:
            if file_.endswith(".json"):
                yield os.path.join(root, file_)


def has_release_version(json_file: Any, keys: Tuple[str, str], version: str) -> bool:
    """Check if the json file has the version in question."""
    rows = (json_file[key] for key in keys)
    return any(version in row for row in rows)


def sorter(key: str) -> str:
    """Since 'main' < 'vX.X.X' and we want to have 'main' as the first entry
    in the list, we return a version that is considerably large. If ibc-go
    reaches this version I'll wear my dunce hat and go sit in the corner.
    """
    return "v99999.9.9" if key == "main" else key


def update_version(json_file: Any, keys: Tuple[str, str], args: argparse.Namespace):
    """Update the versions as required in the json file."""
    recent, new, op = args.recent, args.new, args.type
    for row in (json_file[key] for key in keys):
        if recent not in row:
            continue
        if op == Operation.ADD:
            row.append(new)
            row.sort(key=sorter, reverse=True)
        else:
            index = row.index(recent)
            if op == Operation.REPLACE:
                row[index] = new
            elif op == Operation.REMOVE:
                del row[index]


def version_input(prompt: str, version: Optional[str]) -> str:
    """Input version if not supplied, make it start with a 'v' if it doesn't."""
    if version is None:
        version = input(prompt)
    return version if version.startswith(("v", "V")) else f"v{version}"


def require_version(args: argparse.Namespace):
    """Allow non-required version in argparse but request it if not provided."""
    args.recent = version_input(f"Recent version ({SUGGESTION}): ", args.recent)
    if args.type == Operation.REMOVE:
        return
    args.new = version_input(f"New version ({SUGGESTION}): ", args.new)


def parse_args() -> argparse.Namespace:
    """Parse command line arguments."""
    parser = argparse.ArgumentParser(description="Update JSON files.")
    parser.add_argument(
        "--type",
        choices=[Operation.ADD.name, Operation.REPLACE.name, Operation.REMOVE.name],
        default="ADD",
        help="Type of version update: add a version, replace one or remove one.",
    )
    parser.add_argument(
        "--directory",
        default=DIRECTORY,
        help="Directory path where JSON files are located",
    )
    parser.add_argument(
        "--recent_version",
        dest="recent",
        help=f"Recent version to search in JSON files ({SUGGESTION})",
    )
    parser.add_argument(
        "--new_version",
        dest="new",
        help=f"New version to add in JSON files ({SUGGESTION})",
    )
    parser.add_argument(
        "--verbose",
        "-v",
        action="store_true",
        help="Allow for verbose output",
        default=False,
    )

    args = parser.parse_args()
    args.type = Operation[args.type.upper()]
    require_version(args)
    return args


def print_logs(logs: Dict[str, List[str]], verbose: bool):
    """Print the logs. Verbosity controls if each individual
    file is printed or not.
    """
    updated, skipped = logs["updated"], logs["skipped"]
    if updated:
        if verbose:
            print("Updated files:", *updated, sep="\n - ")
    else:
        print("No files were updated.")
    if skipped:
        if verbose:
            print("The following files were skipped:", *skipped, sep="\n - ")
    else:
        print("No files skipped.")


def main(args: argparse.Namespace):
    """ Main driver function."""
    # Hold logs for 'updated' and 'skipped' files.
    logs = defaultdict(list)
    # Go through each file and operate on it, if applicable.
    for file_ in find_json_files(args.directory):
        with open(file_, "r+") as fp:
            json_file = json.load(fp)
            if not has_release_version(json_file, KEYS, args.recent):
                logs["skipped"].append(
                    f"Version '{args.recent}' not found in '{file_}'"
                )
                continue
            update_version(json_file, KEYS, args)
            fp.seek(0)
            fp.truncate()
            json.dump(json_file, fp, **DUMP_ARGS)
            logs["updated"].append(f"Updated '{file_}'")

    # Print logs collected.
    print_logs(logs, args.verbose)


if __name__ == "__main__":
    args = parse_args()
    main(args)
