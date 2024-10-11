from golden_utils import flatc_golden


def flatc(options, schema):
    # Wrap the golden flatc generator with Dart specifics
    flatc_golden(options=["--dart"] + options, schema=schema, prefix="dart")


def GenerateDart():
    flatc([], "basic.fbs")
