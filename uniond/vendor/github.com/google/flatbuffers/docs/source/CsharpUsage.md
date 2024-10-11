Use in C#    {#flatbuffers_guide_use_c-sharp}
==============

## Before you get started

Before diving into the FlatBuffers usage in C#, it should be noted that
the [Tutorial](@ref flatbuffers_guide_tutorial) page has a complete guide to
general FlatBuffers usage in all of the supported languages (including C#).
This page is designed to cover the nuances of FlatBuffers usage,
specific to C#.

You should also have read the [Building](@ref flatbuffers_guide_building)
documentation to build `flatc` and should be familiar with
[Using the schema compiler](@ref flatbuffers_guide_using_schema_compiler) and
[Writing a schema](@ref flatbuffers_guide_writing_schema).

## FlatBuffers C# code location

The code for the FlatBuffers C# library can be found at
`flatbuffers/net/FlatBuffers`. You can browse the library on the
[FlatBuffers GitHub page](https://github.com/google/flatbuffers/tree/master/net/
FlatBuffers).

## Building the FlatBuffers C# library

The `FlatBuffers.csproj` project contains multitargeting for .NET Standard 2.1,
.NET 6 and .NET 8.

You can build for a specific framework target when using the cross-platform
[.NET Core SDK](https://dotnet.microsoft.com/download) by adding the `-f`
command line option:

~~~{.sh}
    dotnet build -f netstandard2.1 "FlatBuffers.csproj"
~~~

The `FlatBuffers.csproj` project also provides support for defining various
conditional compilation symbols (see "Conditional compilation symbols" section
below) using the `-p` command line option:

~~~{.sh}
    dotnet build -f netstandard2.1 -p:ENABLE_SPAN_T=true -p:UNSAFE_BYTEBUFFER=true "FlatBuffers.csproj"
~~~

## Testing the FlatBuffers C# library

The code to test the libraries can be found at `flatbuffers/tests`.

The test code for C# is located in the [FlatBuffers.Test](https://github.com/
google/flatbuffers/tree/master/tests/FlatBuffers.Test) subfolder. To run the
tests, open `FlatBuffers.Test.csproj` in [Visual Studio](
https://www.visualstudio.com), and compile/run the project.

Optionally, you can run this using [Mono](http://www.mono-project.com/) instead.
Once you have installed Mono, you can run the tests from the command line
by running the following commands from inside the `FlatBuffers.Test` folder:

~~~{.sh}
    mcs *.cs ../MyGame/Example/*.cs ../../net/FlatBuffers/*.cs
    mono Assert.exe
~~~

## Using the FlatBuffers C# library

*Note: See [Tutorial](@ref flatbuffers_guide_tutorial) for a more in-depth
example of how to use FlatBuffers in C#.*

FlatBuffers supports reading and writing binary FlatBuffers in C#.

To use FlatBuffers in your own code, first generate C# classes from your
schema with the `--csharp` option to `flatc`.
Then you can include both FlatBuffers and the generated code to read
or write a FlatBuffer.

For example, here is how you would read a FlatBuffer binary file in C#:
First, import the library and generated code. Then, you read a FlatBuffer binary
file into a `byte[]`.  You then turn the `byte[]` into a `ByteBuffer`, which you
pass to the `GetRootAsMyRootType` function:

~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~{.cs}
    using MyGame.Example;
    using Google.FlatBuffers;

    // This snippet ignores exceptions for brevity.
    byte[] data = File.ReadAllBytes("monsterdata_test.mon");

    ByteBuffer bb = new ByteBuffer(data);
    Monster monster = Monster.GetRootAsMonster(bb);
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Now you can access the data from the `Monster monster`:

~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~{.cs}
    short hp = monster.Hp;
    Vec3 pos = monster.Pos;
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

C# code naming follows standard C# style with PascalCasing identifiers,
e.g. `GetRootAsMyRootType`. Also, values (except vectors and unions) are
available as properties instead of parameterless accessor methods.
The performance-enhancing methods to which you can pass an already created
object are prefixed with `Get`, e.g.:

~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~{.cs}
    // property
    var pos = monster.Pos;

    // method filling a preconstructed object
    var preconstructedPos = new Vec3();
    monster.GetPos(preconstructedPos);
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

## Storing dictionaries in a FlatBuffer

FlatBuffers doesn't support dictionaries natively, but there is support to
emulate their behavior with vectors and binary search, which means you
can have fast lookups directly from a FlatBuffer without having to unpack
your data into a `Dictionary` or similar.

To use it:
-   Designate one of the fields in a table as the "key" field. You do this
    by setting the `key` attribute on this field, e.g.
    `name:string (key)`.
    You may only have one key field, and it must be of string or scalar type.
-   Write out tables of this type as usual, collect their offsets in an
    array.
-   Instead of calling standard generated method,
    e.g.: `Monster.createTestarrayoftablesVector`,
    call `CreateSortedVectorOfMonster` in C#
    which will first sort all offsets such that the tables they refer to
    are sorted by the key field, then serialize it.
-   Now when you're accessing the FlatBuffer, you can use
    the `ByKey` accessor to access elements of the vector, e.g.:
    `monster.TestarrayoftablesByKey("Frodo")` in C#,
    which returns an object of the corresponding table type,
    or `null` if not found.
    `ByKey` performs a binary search, so should have a similar
    speed to `Dictionary`, though may be faster because of better caching.
    `ByKey` only works if the vector has been sorted, it will
    likely not find elements if it hasn't been sorted.

## Buffer verification

As mentioned in [C++ Usage](@ref flatbuffers_guide_use_cpp) buffer
accessor functions do not verify buffer offsets at run-time.
If it is necessary, you can optionally use a buffer verifier before you
access the data. This verifier will check all offsets, all sizes of
fields, and null termination of strings to ensure that when a buffer
is accessed, all reads will end up inside the buffer.

Each root type will have a verification function generated for it,
e.g. `Monster.VerifyMonster`. This can be called as shown:
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~{.cs}
    var ok = Monster.VerifyMonster(buf);
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
if `ok` is true, the buffer is safe to read.

For a more detailed control of verification `MonsterVerify.Verify`
for `Monster` type can be used:
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~{.cs}
    # Sequence of calls
    FlatBuffers.Verifier verifier = new FlatBuffers.Verifier(buf);
    var ok = verifier.VerifyBuffer("MONS", false, MonsterVerify.Verify);

    # Or single line call
    var ok = new FlatBuffers.Verifier(bb).setStringCheck(true).\
             VerifyBuffer("MONS", false, MonsterVerify.Verify);

~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
if `ok` is true, the buffer is safe to read.

A second parameter of `verifyBuffer` specifies whether buffer content is
size prefixed or not. In the example above, the buffer is assumed to not include
size prefix (`false`).

Verifier supports options that can be set using appropriate fluent methods:
* SetMaxDepth - limit the nesting depth. Default: 1000000
* SetMaxTables - total amount of tables the verifier may encounter. Default: 64
* SetAlignmentCheck - check content alignment. Default: True
* SetStringCheck - check if strings contain termination '0' character. Default: true


## Text parsing

There currently is no support for parsing text (Schema's and JSON) directly
from C#, though you could use the C++ parser through native call
interfaces available to each language. Please see the
C++ documentation for more on text parsing.

## Object based API

FlatBuffers is all about memory efficiency, which is why its base API is written
around using as little as possible of it. This does make the API clumsier
(requiring pre-order construction of all data, and making mutation harder).

For times when efficiency is less important a more convenient object based API
can be used (through `--gen-object-api`) that is able to unpack & pack a
FlatBuffer into objects and standard `System.Collections.Generic` containers,
allowing for convenient construction, access and mutation.

To use:

~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~{.cs}
    // Deserialize from buffer into object.
    MonsterT monsterobj = GetMonster(flatbuffer).UnPack();

    // Update object directly like a C# class instance.
    Console.WriteLine(monsterobj.Name);
    monsterobj.Name = "Bob";  // Change the name.

    // Serialize into new flatbuffer.
    FlatBufferBuilder fbb = new FlatBufferBuilder(1);
    fbb.Finish(Monster.Pack(fbb, monsterobj).Value);
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

### Json Serialization

An additional feature of the object API is the ability to allow you to
serialize & deserialize a JSON text.
To use Json Serialization, add `--cs-gen-json-serializer` option to `flatc` and
add `Newtonsoft.Json` nuget package to csproj. This requires explicitly setting
the `--gen-object-api` option as well.

~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~{.cs}
    // Deserialize MonsterT from json
    string jsonText = File.ReadAllText(@"Resources/monsterdata_test.json");
    MonsterT mon = MonsterT.DeserializeFromJson(jsonText);

    // Serialize MonsterT to json
    string jsonText2 = mon.SerializeToJson();
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

* Limitation
  * `hash` attribute currently not supported.
* NuGet package Dependency
  * [Newtonsoft.Json](https://github.com/JamesNK/Newtonsoft.Json)

## Conditional compilation symbols

There are three conditional compilation symbols that have an impact on
performance/features of the C# `ByteBuffer` implementation.

* `UNSAFE_BYTEBUFFER`

  This will use unsafe code to manipulate the underlying byte array. This can
  yield a reasonable performance increase.

* `BYTEBUFFER_NO_BOUNDS_CHECK`

  This will disable the bounds check asserts to the byte array. This can yield a
  small performance gain in normal code.

* `ENABLE_SPAN_T`

  This will enable reading and writing blocks of memory with a `Span<T>` instead
  of just `T[]`. You can also enable writing directly to shared memory or other
  types of memory by providing a custom implementation of `ByteBufferAllocator`.
  `ENABLE_SPAN_T` also requires `UNSAFE_BYTEBUFFER` to be defined, or .NET
  Standard 2.1.

Using `UNSAFE_BYTEBUFFER` and `BYTEBUFFER_NO_BOUNDS_CHECK` together can yield a
performance gain of ~15% for some operations, however doing so is potentially
dangerous. Do so at your own risk!

<br>
