discard """
  action:   "run"
  exitcode: 0
  timeout:  60.0
"""
import std/unittest
import std/options
import flatbuffers
import ../../../optional_scalars/ScalarStuff


suite "TestOptionalScalars":

  test "OptionalScalars":
    var builder = newBuilder(1024)
    builder.ScalarStuffStart()
    let root = builder.ScalarStuffEnd()
    builder.Finish(root)

    var optionals: ScalarStuff
    optionals.GetRootAs(builder.FinishedBytes(), 0)

    # Creates a flatbuffer with optional values.
    check(optionals.justI8 == 0)
    check(optionals.maybeF32.isNone)
    check(optionals.defaultBool == true)
    check(optionals.justU16 == 0)
    check(optionals.maybeEnum.isNone)
    check(optionals.defaultU64 == 42)
