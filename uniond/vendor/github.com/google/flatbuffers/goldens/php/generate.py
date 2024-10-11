from golden_utils import flatc_golden


def flatc(options, schema):
    # Wrap the golden flatc generator with PHP specifics
    flatc_golden(options=["--php"] + options, schema=schema, prefix="php")


def GeneratePhp():
    flatc([], "basic.fbs")
