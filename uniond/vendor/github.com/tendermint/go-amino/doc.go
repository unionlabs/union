/*

Amino is an encoding library that can handle interfaces (like protobuf "oneof")
well.  This is achieved by prefixing bytes before each "concrete type".

A concrete type is some non-interface value (generally a struct) which
implements the interface to be (de)serialized. Not all structures need to be
registered as concrete types -- only when they will be stored in interface type
fields (or interface type slices) do they need to be registered.


Registering types

All interfaces and the concrete types that implement them must be registered.

 amino.RegisterInterface((*MyInterface1)(nil), nil)
 amino.RegisterInterface((*MyInterface2)(nil), nil)
 amino.RegisterConcrete(MyStruct1{}, "com.tendermint/MyStruct1", nil)
 amino.RegisterConcrete(MyStruct2{}, "com.tendermint/MyStruct2", nil)
 amino.RegisterConcrete(&MyStruct3{}, "anythingcangoinhereifitsunique", nil)

Notice that an interface is represented by a nil pointer.

Structures that must be deserialized as pointer values must be registered with
a pointer value as well.  It's OK to (de)serialize such structures in
non-pointer (value) form, but when deserializing such structures into an
interface field, they will always be deserialized as pointers.


How it works

All registered concrete types are encoded with leading 4 bytes (called "prefix
bytes"), even when it's not held in an interface field/element.  In this way,
Amino ensures that concrete types (almost) always have the same canonical
representation.  The first byte of the prefix bytes must not be a zero byte, so
there are 2**(8*4)-2**(8*3) possible values.

When there are 4096 types registered at once, the probability of there being a
conflict is ~ 0.2%. See https://instacalc.com/51189 for estimation.  This is
assuming that all registered concrete types have unique natural names (e.g.
prefixed by a unique entity name such as "com.tendermint/", and not
"mined/grinded" to produce a particular sequence of "prefix bytes").

TODO Update instacalc.com link with 255/256 since 0x00 is an escape.

Do not mine/grind to produce a particular sequence of prefix bytes, and avoid
using dependencies that do so.

Since 4 bytes are not sufficient to ensure no conflicts, sometimes it is
necessary to prepend more than the 4 prefix bytes for disambiguation.  Like the
prefix bytes, the disambiguation bytes are also computed from the registered
name of the concrete type.  There are 3 disambiguation bytes, and in binary
form they always precede the prefix bytes.  The first byte of the
disambiguation bytes must not be a zero byte, so there are 2**(8*3)-2**(8*2)
possible values.

 // Sample Amino encoded binary bytes with 4 prefix bytes.
 > [0xBB 0x9C 0x83 0xDD] [...]

 // Sample Amino encoded binary bytes with 3 disambiguation bytes and 4
 // prefix bytes.
 > 0x00 <0xA8 0xFC 0x54> [0xBB 0x9C 0x83 0xDD] [...]

The prefix bytes never start with a zero byte, so the disambiguation bytes are
escaped with 0x00.

Notice that the 4 prefix bytes always immediately precede the binary encoding
of the concrete type.


Computing prefix bytes

To compute the disambiguation bytes, we take `hash := sha256(concreteTypeName)`,
and drop the leading 0x00 bytes.

 > hash := sha256("com.tendermint.consensus/MyConcreteName")
 > hex.EncodeBytes(hash) // 0x{00 00 A8 FC 54 00 00 00 BB 9C 83 DD ...} (example)

In the example above, hash has two leading 0x00 bytes, so we drop them.

 > rest = dropLeadingZeroBytes(hash) // 0x{A8 FC 54 00 00 BB 9C 83 DD ...}
 > disamb = rest[0:3]
 > rest = dropLeadingZeroBytes(rest[3:])
 > prefix = rest[0:4]

The first 3 bytes are called the "disambiguation bytes" (in angle brackets).
The next 4 bytes are called the "prefix bytes" (in square brackets).

 > <0xA8 0xFC 0x54> [0xBB 0x9C 9x83 9xDD]

*/
package amino
