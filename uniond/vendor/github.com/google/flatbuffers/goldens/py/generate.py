from golden_utils import flatc_golden


def flatc(options, schema):
    # Wrap the golden flatc generator with Python specifics
    flatc_golden(options=["--python"] + options, schema=schema, prefix="py")


def GeneratePython():
    flatc([], "basic.fbs")
