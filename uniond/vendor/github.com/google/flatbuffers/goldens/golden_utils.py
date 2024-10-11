import sys
from pathlib import Path

# Get the path where this script is located so we can invoke the script from
# any directory and have the paths work correctly.
script_path = Path(__file__).parent.resolve()

# Get the root path as an absolute path, so all derived paths are absolute.
root_path = script_path.parent.absolute()

# Get the location of the schema
schema_path = Path(script_path, "schema")

# Too add the util package in /scripts/util.py
sys.path.append(str(root_path.absolute()))

from scripts.util import flatc


def flatc_golden(options, schema, prefix):
    # wrap the generic flatc call with specifis for these goldens.
    flatc(
        options=options,
        # where the files are generated, typically the language (e.g. "cpp").
        prefix=prefix,
        # The schema are relative to the schema directory.
        schema=str(Path(schema_path, schema)),
        # Run flatc from this location.
        cwd=script_path,
    )
