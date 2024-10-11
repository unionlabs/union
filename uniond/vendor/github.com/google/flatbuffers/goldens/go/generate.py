from golden_utils import flatc_golden


def flatc(options, schema):
    # Wrap the golden flatc generator with Go specifics
    flatc_golden(options=["--go"] + options, schema=schema, prefix="go")


def GenerateGo():
    flatc([], "basic.fbs")
