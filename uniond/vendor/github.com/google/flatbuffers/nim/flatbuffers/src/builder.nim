import math
import table


const MAX_BUFFER_SIZE* = 2^31


type Builder* = ref object of RootObj
   bytes*: seq[byte]
   minalign*: int
   current_vtable*: seq[uoffset]
   objectEnd*: uoffset
   vtables*: seq[uoffset] #?
   head*: uoffset
   nested*: bool
   finished*: bool
   vectorNumElems*: uoffset

using this: var Builder

func newBuilder*(size: int): Builder =
   result = new Builder
   result.bytes.setLen(size)
   result.minalign = 1
   result.head = size.uoffset
   result.nested = false
   result.finished = false
   result.vectorNumElems = 0

proc FinishedBytes*(this): seq[byte] =
   if not this.finished:
      quit("Builder not finished, Incorrect use of FinishedBytes(): must call 'Finish' first.")
   result = this.bytes[this.head..^1]

proc Output*(this): seq[byte] =
   if not this.finished:
      quit("Builder not finished, Incorrect use of Output(): must call 'Finish' first.")

   result = this.bytes[this.head..^1]

func Offset*(this): uoffset =
   result = this.bytes.len.uoffset - this.head

proc StartObject*(this; numfields: int) =
   if this.nested:
      quit("builder is nested")

   this.current_vtable.setLen(numfields)
   for i in this.current_vtable.mitems():
      i = 0
   this.objectEnd = this.Offset()
   this.nested = true

proc GrowByteBuffer*(this) =
   if this.bytes.len == MAX_BUFFER_SIZE:
      quit("flatbuffers: cannot grow buffer beyond 2 gigabytes")
   let oldLen = this.bytes.len
   var newLen = min(this.bytes.len * 2, MAX_BUFFER_SIZE)
   if newLen == 0:
      newLen = 1
   this.bytes.setLen(newLen)
   var j = this.bytes.len - 1
   while j >= 0:
      if j >= newLen - oldLen:
         this.bytes[j] = this.bytes[j - (newLen - oldLen)]
      else:
         this.bytes[j] = 0
      dec(j)

proc Place*[T](this; x: T) =
   this.head -= uoffset x.sizeof
   WriteVal(this.bytes, this.head, x)

func Pad*(this; n: int) =
   for i in 0..<n:
      this.Place(0.byte)

proc Prep*(this; size: int; additionalBytes: int) =
   if size > this.minalign:
      this.minalign = size
   var alignsize = (not (this.bytes.len - this.head.int + additionalBytes)) + 1
   alignsize = alignsize and (size - 1)

   while this.head.int < alignsize + size + additionalBytes:
      let oldbufSize = this.bytes.len
      this.GrowByteBuffer()
      this.head = (this.head.int + this.bytes.len - oldbufSize).uoffset
   this.Pad(alignsize)

proc PrependOffsetRelative*[T: Offsets](this; off: T) =
   when T is voffset:
      this.Prep(T.sizeof, 0)
      if not off.uoffset <= this.Offset:
         quit("flatbuffers: Offset arithmetic error.")
      this.Place(off)
   else:
      this.Prep(T.sizeof, 0)
      if not off.uoffset <= this.Offset:
         quit("flatbuffers: Offset arithmetic error.")
      let off2: T = this.Offset.T - off + sizeof(T).T
      this.Place(off2)


proc Prepend*[T](this; x: T) =
   this.Prep(x.sizeof, 0)
   this.Place(x)

proc Slot*(this; slotnum: int) =
   this.current_vtable[slotnum] = this.Offset

proc PrependSlot*[T](this; o: int; x, d: T) =
   if x != d:
      when T is uoffset or T is soffset or T is voffset:
         this.PrependOffsetRelative(x)
      else:
         this.Prepend(x)
      this.Slot(o)

proc AssertStuctInline(this; obj: uoffset) =
   if obj != this.Offset:
      quit("flatbuffers: Tried to write a Struct at an Offset that is different from the current Offset of the Builder.")

proc PrependStructSlot*(this; o: int; x: uoffset; d: uoffset) =
   if x != d:
      this.AssertStuctInline(x)
      this.Slot(o)

proc Add*[T](this; n: T) =
   this.Prep(T.sizeof, 0)
   WriteVal(this.bytes, this.head, n)

proc VtableEqual*(a: seq[uoffset]; objectStart: uoffset; b: seq[byte]): bool =
   if a.len * voffset.sizeof != b.len:
      return false

   var i = 0
   while i < a.len:
      var seq = b[i * voffset.sizeof..<(i + 1) * voffset.sizeof]
      let x = GetVal[voffset](addr seq)

      if x == 0 and a[i] == 0:
         inc i
         continue

      let y = objectStart.soffset - a[i].soffset
      if x.soffset != y:
         return false
      inc i
   return true

proc WriteVtable*(this): uoffset =
   this.PrependOffsetRelative(0.soffset)

   let objectOffset = this.Offset
   var existingVtable = uoffset 0

   var i = this.current_vtable.len - 1
   while i >= 0 and this.current_vtable[i] == 0: dec i

   this.current_vtable = this.current_vtable[0..i]
   for i in countdown(this.vtables.len - 1, 0):
      let
         vt2Offset: uoffset = this.vtables[i]
         vt2Start: int = this.bytes.len - int vt2Offset

      var seq = this.bytes[vt2Start..<this.bytes.len]
      let
         vt2Len = GetVal[voffset](addr seq)
         metadata = 2 * voffset.sizeof # VtableMetadataFields * SizeVOffsetT
         vt2End = vt2Start + vt2Len.int
         vt2 = this.bytes[this.bytes.len - vt2Offset.int + metadata..<vt2End]

      if VtableEqual(this.current_vtable, objectOffset, vt2):
         existingVtable = vt2Offset
         break

   if existingVtable == 0:
      for i in countdown(this.current_vtable.len - 1, 0):
         var off: uoffset
         if this.current_vtable[i] != 0:
            off = objectOffset - this.current_vtable[i]

         this.PrependOffsetRelative(off.voffset)

      let objectSize = objectOffset - this.objectEnd
      this.PrependOffsetRelative(objectSize.voffset)

      let vBytes = (this.current_vtable.len + 2) * voffset.sizeof
      this.PrependOffsetRelative(vBytes.voffset)

      let objectStart: uoffset = (this.bytes.len.uoffset - objectOffset)
      WriteVal(this.bytes, objectStart, (this.Offset - objectOffset).soffset)
      this.vtables.add this.Offset
   else:
      let objectStart: uoffset = this.bytes.len.uoffset - objectOffset
      this.head = objectStart
      WriteVal(this.bytes, this.head,
         (existingVtable.soffset - objectOffset.soffset))

      this.current_vtable = @[]
   result = objectOffset

proc EndObject*(this): uoffset =
   if not this.nested:
      quit("builder is not nested")
   result = this.WriteVtable()
   this.nested = false

proc End*(this: var Builder): uoffset =
   result = this.EndObject()

proc StartVector*(this; elemSize: int; numElems: uoffset;
      alignment: int) =
   if this.nested:
      quit("builder is nested")
   this.nested = true
   this.vectorNumElems = numElems
   this.Prep(sizeof(uint32), elemSize * numElems.int)
   this.Prep(alignment, elemSize * numElems.int)

proc EndVector*(this): uoffset =
   if not this.nested:
      quit("builder is not nested")
   this.nested = false
   this.Place(this.vectorNumElems)
   this.vectorNumElems = 0
   result = this.Offset

proc getChars*(str: seq[byte]): string =
   var bytes = str
   result = GetVal[string](addr bytes)

proc getBytes*(str: string | cstring): seq[byte] =
   for chr in str:
      result.add byte chr
   result.add byte 0

proc Create*[T](this; s: T): uoffset = # Both CreateString and CreateByteVector functionality
   if this.nested:
      quit("builder is nested")
   this.nested = true
   when T is cstring or T is string:
      let x = s.getBytes()
      let l = x.len.uoffset
      this.vectorNumElems = l-1
   else:
      let x = s
      let l = x.len.uoffset
      this.vectorNumElems = l
   this.Prep(uoffset.sizeof, l.int * byte.sizeof)
   this.head -= l
   this.bytes[this.head..<this.head+l] = x
   result = this.EndVector()

proc Finish*(this; rootTable: uoffset) =
   if this.nested:
      quit("builder is nested")
   this.nested = true

   this.Prep(this.minalign, uoffset.sizeof)
   this.PrependOffsetRelative(rootTable)
   this.finished = true
