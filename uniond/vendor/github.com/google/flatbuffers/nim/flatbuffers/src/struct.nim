import table


type FlatObj* {.inheritable.} = object
   tab*: Vtable

func Table*(this: var FlatObj): Vtable = this.tab

func Init*(this: var FlatObj; buf: seq[byte]; i: uoffset) =
   this.tab.Bytes = buf
   this.tab.Pos = i

# Cant define it in table.nim since it needs FlatObj and Init
func GetUnion*[T: FlatObj](this: var Vtable; off: uoffset): T =
   result.Init(this.Bytes, this.Indirect(off))

func GetRootAs*(result: var FlatObj; buf: seq[byte]; offset: uoffset) =
   var
      vtable = Vtable(Bytes: buf[offset..^1], Pos: offset)
      n = Get[uoffset](vtable, offset)
   result.Init(buf, n+offset)

func GetRootAs*(result: var FlatObj; buf: string; offset: uoffset) =
   result.GetRootAs(cast[seq[byte]](buf), offset)
