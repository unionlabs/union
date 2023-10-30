organization := "org.zfnd"

name := "ed25519jni"

version := "0.0.4-JNI-DEV"

scalaVersion := "2.12.10"

scalacOptions ++= Seq("-Xmax-classfile-name", "140")

autoScalaLibrary := false // exclude scala-library from dependencies

crossPaths := false // drop off Scala suffix from artifact names.

libraryDependencies ++= Deps.ed25519jni

unmanagedResourceDirectories in Compile += baseDirectory.value / "natives"

publishArtifact := true

javacOptions in (Compile,doc) ++= Seq(
  "-windowtitle", "JNI bindings for ed25519-zebra"
)

testOptions in Test += Tests.Argument(TestFrameworks.ScalaCheck, "-verbosity", "3")
