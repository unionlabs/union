from golden_utils import flatc_golden


def flatc(options, schema):
    # Wrap the golden flatc generator with C# specifics
    flatc_golden(options=["--csharp"] + options, schema=schema, prefix="csharp")


def GenerateCSharp():
    flatc([], "basic.fbs")
