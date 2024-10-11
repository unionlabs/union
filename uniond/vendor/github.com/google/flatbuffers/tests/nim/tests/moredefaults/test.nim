discard """
  action:   "run"
  exitcode: 0
  timeout:  60.0
"""
import std/unittest
import flatbuffers
import ../../../MoreDefaults

suite "TestMoreDefaults":

  test "testFlatbuffersObject":
    var fbb = newBuilder(0)
    fbb.MoreDefaultsStart()
    let root = fbb.MoreDefaultsEnd()
    fbb.Finish(root)

    var defaults: MoreDefaults
    defaults.GetRootAs(fbb.FinishedBytes(), 0)
    check(defaults.emptyString == "")
    check(defaults.ints == [])
    check(defaults.floats == [])
    check(defaults.bools == [])
    check(defaults.intsLength == 0)
    check(defaults.floatsLength == 0)
    check(defaults.abcsLength == 0)
    check(defaults.boolsLength == 0)
