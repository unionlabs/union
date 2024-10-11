from golden_utils import flatc_golden


def flatc(options, schema):
    # Wrap the golden flatc generator with Lobster specifics
    flatc_golden(options=["--lobster"] + options, schema=schema, prefix="lobster")


def GenerateLobster():
    flatc([], "basic.fbs")
