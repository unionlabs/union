from golden_utils import flatc_golden


def flatc(options, schema):
    # Wrap the golden flatc generator with Swift specifics
    flatc_golden(options=["--swift"] + options, schema=schema, prefix="swift")


def GenerateSwift():
    flatc([], "basic.fbs")
