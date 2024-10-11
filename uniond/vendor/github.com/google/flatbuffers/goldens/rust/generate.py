from golden_utils import flatc_golden


def flatc(options, schema):
    # Wrap the golden flatc generator with Rust specifics
    flatc_golden(options=["--rust"] + options, schema=schema, prefix="rust")


def GenerateRust():
    flatc([], "basic.fbs")
