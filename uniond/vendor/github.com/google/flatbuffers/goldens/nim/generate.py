from golden_utils import flatc_golden


def flatc(options, schema):
    # Wrap the golden flatc generator with Nim specifics
    flatc_golden(options=["--nim"] + options, schema=schema, prefix="nim")


def GenerateNim():
    flatc([], "basic.fbs")
