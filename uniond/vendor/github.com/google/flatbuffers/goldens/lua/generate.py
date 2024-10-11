from golden_utils import flatc_golden


def flatc(options, schema):
    # Wrap the golden flatc generator with Lua specifics
    flatc_golden(options=["--lua"] + options, schema=schema, prefix="lua")


def GenerateLua():
    flatc([], "basic.fbs")
