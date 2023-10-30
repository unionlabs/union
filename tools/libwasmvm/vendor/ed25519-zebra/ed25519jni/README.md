# JNI
Code that provides a [JNI](https://en.wikipedia.org/wiki/Java_Native_Interface)
for the library is included. Allows any JNI-using language to interact with
specific `ed25519-zebra` calls and provides a minor analogue for some Rust
classes, allowing for things like basic sanity checks of certain values.  Tests
written in Scala have also been included.

## Compilation / Library Usage
To build the JNI code, there are several steps. The exact path forward depends
on the user's preferred deployment method. No matter what, the following steps
must be performed at the beginning.

- Run `cargo build` in the root directory. This generates the core Rust code.
- Run `cargo build` in the `ed25519jni/rust` subdirectory. This generates the Rust
  glue code libraries (`libed25519jni.a` and `libed25519jni.{so/dylib}`).

From here, there are two deployment methods: Direct library usage and JARs.

### JAR
<a name="jar"></a>

It's possible to generate a JAR that can be loaded into a project via
[SciJava's NativeLoader](https://javadoc.scijava.org/SciJava/org/scijava/nativelib/NativeLoader.html),
along with the Java JNI interface file. There are two exta steps to perform
after the mandatory compilation steps.

- Run `jni_jar_prereq.sh` from the `ed25519/scripts` subdirectory. This performs
  some JAR setup steps.
- Run `sbt clean publishLocal` from the `ed25519jni/jvm` subdirectory. This
  generates the final `ed25519jni.jar` file.

### Direct library usage
(NOTE: Future work will better accommodate this option. For now, users will have
to develop their own solutions.)

Use a preferred method to load the Rust core and JNI libraries directly as
needed. If necessary, include the JNI Java files too.

## Testing
Run `sbt test` from the `ed25519jni/jvm` directory. Note that, in order to run
the tests, the [JAR compilation method](#jar) must be executed first.

## Capabilities
Among other things, the JNI code can perform the following actions.

* Generate a random 32 byte signing key seed.
* Generate a 32 byte verification key from a signing key seed.
* Sign arbitrary data with a signing key seed.
* Verify a signature for arbitrary data with verification key bytes (32 bytes).
