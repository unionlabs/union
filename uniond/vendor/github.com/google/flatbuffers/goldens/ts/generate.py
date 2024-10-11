from golden_utils import flatc_golden


def flatc(options, schema):
    # Wrap the golden flatc generator with Swift specifics
    flatc_golden(options=["--ts"] + options, schema=schema, prefix="ts")


def GenerateTs():
    flatc([], "basic.fbs")
