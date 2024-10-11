Use in Swift {#flatbuffers_guide_use_swift}
=========

## Before you get started

Before diving into the FlatBuffers usage in Swift, it should be noted that
the [Tutorial](@ref flatbuffers_guide_tutorial) page has a complete guide
to general FlatBuffers usage in all of the supported languages (including Swift).
This page is designed to cover the nuances of FlatBuffers usage, specific to
Swift.

You should also have read the [Building](@ref flatbuffers_guide_building)
documentation to build `flatc` and should be familiar with
[Using the schema compiler](@ref flatbuffers_guide_using_schema_compiler) and
[Writing a schema](@ref flatbuffers_guide_writing_schema).

## FlatBuffers Swift library code location

The code for the FlatBuffers Swift library can be found at
`flatbuffers/swift`. You can browse the library code on the [FlatBuffers
GitHub page](https://github.com/google/flatbuffers/tree/master/swift).

## Testing the FlatBuffers Swift library

The code to test the Swift library can be found at `flatbuffers/tests/swift/tests`.
The test code itself is located in [flatbuffers/tests/swift/tests](https://github.com/google/flatbuffers/blob/master/tests/swift/tests).

To run the tests, use the [SwiftTest.sh](https://github.com/google/flatbuffers/blob/master/tests/swift/tests/SwiftTest.sh) shell script.

*Note: The shell script requires [Swift](https://swift.org) to
be installed.*

## Using the FlatBuffers Swift library

*Note: See [Tutorial](@ref flatbuffers_guide_tutorial) for a more in-depth
example of how to use FlatBuffers in Swift.*

FlatBuffers supports reading and writing binary FlatBuffers in Swift.

To use FlatBuffers in your own code, first generate Swift structs from your
schema with the `--swift` option to `flatc`. Then include FlatBuffers using `SPM` in
by adding the path to `FlatBuffers/swift` into it. The generated code should also be
added to xcode or the path of the package you will be using. Note: sometimes xcode cant
and wont see the generated files, so it's better that you copy them to xcode.

For example, here is how you would read a FlatBuffer binary file in Swift: First,
include the library and copy thegenerated code. Then read a FlatBuffer binary file or
a data object from the server, which you can pass into the `GetRootAsMonster` function.

~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~{.swift}
    import FlatBuffers

    typealias Monster1 = MyGame.Sample.Monster
    typealias Vec3 = MyGame.Sample.Vec3

    let path = FileManager.default.currentDirectoryPath
    let url = URL(fileURLWithPath: path, isDirectory: true).appendingPathComponent("monsterdata_test").appendingPathExtension("mon")
    guard let data = try? Data(contentsOf: url) else { return }

    let monster = Monster.getRootAsMonster(bb: ByteBuffer(data: data))
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Now you can access values like this:

~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~{.swift}
    let hp = monster.hp
    let pos = monster.pos // uses native swift structs
    let pos = monster.mutablePos // uses flatbuffers structs
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~


In some cases it's necessary to modify values in an existing FlatBuffer in place (without creating a copy). For this reason, scalar fields of a Flatbuffer table or struct can be mutated.

~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~{.swift}
    var byteBuffer = ByteBuffer(bytes: data)
    // Get an accessor to the root object inside the buffer.
    let monster: Monster = try! getCheckedRoot(byteBuffer: &byteBuffer)
    // let monster: Monster = getRoot(byteBuffer: &byteBuffer)

    if !monster.mutate(hp: 10) {
      fatalError("couldn't mutate")
    }
    // mutate a struct field using flatbuffers struct
    // DONT use monster.pos to mutate since swift copy on write 
    // will not mutate the value in the buffer
    let vec = monster.mutablePos.mutate(z: 4)

    // This mutation will fail because the mana field is not available in
    // the buffer. It should be set when creating the buffer.
    if !monster.mutate(mana: 20) {
      fatalError("couldn't mutate")
    }
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The term `mutate` is used instead of `set` to indicate that this is a special use case. All mutate functions return a boolean value which is false if the field we're trying to mutate is not available in the buffer.

<br>
