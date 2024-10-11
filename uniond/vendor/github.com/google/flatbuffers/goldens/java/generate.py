from golden_utils import flatc_golden


def flatc(options, schema):
    # Wrap the golden flatc generator with Java specifics
    flatc_golden(options=["--java"] + options, schema=schema, prefix="java")


def GenerateJava():
    flatc([], "basic.fbs")
