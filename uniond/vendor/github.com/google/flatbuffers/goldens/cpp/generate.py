from golden_utils import flatc_golden


def flatc(options, schema):
    # Wrap the golden flatc generator with C++ specifics
    flatc_golden(options=["--cpp"] + options, schema=schema, prefix="cpp")


def GenerateCpp():
    flatc([], "basic.fbs")
