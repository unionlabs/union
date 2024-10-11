template swapEndian*(outp, inp: pointer, size: int) =
   var i = cast[cstring](inp)
   var o = cast[cstring](outp)
   for x in 0..<size:
      o[x] = i[(0..<size).len - x - 1]

when system.cpuEndian == bigEndian:
   func littleEndianX*(outp, inp: pointer, size: int) {.inline.} = swapEndian(outp, inp, size)
   func bigEndianX*(outp, inp: pointer, size: int) {.inline.} = copyMem(outp, inp, size)
else:
   func littleEndianX*(outp, inp: pointer, size: int) {.inline.} = copyMem(outp, inp, size)
   func bigEndianX*(outp, inp: pointer, size: int) {.inline.} = swapEndian(outp, inp, size)
