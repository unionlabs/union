from golden_utils import flatc_golden


def flatc(options, schema):
    # Wrap the golden flatc generator with Kotlin specifics
    flatc_golden(options=["--kotlin"] + options, schema=schema, prefix="kotlin")


def GenerateKotlin():
    flatc([], "basic.fbs")
