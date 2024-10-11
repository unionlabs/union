import groovy.xml.XmlParser

plugins {
  kotlin("multiplatform")
  id("org.jetbrains.kotlinx.benchmark")
  id("io.morethan.jmhreport")
  id("de.undercouch.download")
}

group = "com.google.flatbuffers.jmh"
version = "2.0.0-SNAPSHOT"

// Reads latest version from Java's runtime pom.xml,
// so we can use it for benchmarking against Kotlin's
// runtime
fun readJavaFlatBufferVersion(): String {
  val pom = XmlParser().parse(File("../java/pom.xml"))
  val versionTag = pom.children().find {
    val node = it as groovy.util.Node
    node.name().toString().contains("version")
  } as groovy.util.Node
  return versionTag.value().toString()
}

// This plugin generates a static html page with the aggregation
// of all benchmarks ran. very useful visualization tool.
jmhReport {
  val baseFolder = project.file("build/reports/benchmarks/main").absolutePath
  val lastFolder = project.file(baseFolder).list()?.sortedArray()?.lastOrNull() ?: ""
  jmhResultPath = "$baseFolder/$lastFolder/jvm.json"
  jmhReportOutput = "$baseFolder/$lastFolder"
}

// For now we benchmark on JVM only
benchmark {
  configurations {
    this.getByName("main") {
      iterations = 5
      iterationTime = 300
      iterationTimeUnit = "ms"
      // uncomment for benchmarking JSON op only
       include(".*FlatbufferBenchmark.*")
    }
  }
  targets {
    register("jvm")
  }
}

kotlin {
  jvm {
    compilations {
      val main by getting { }
      // custom benchmark compilation
      val benchmarks by compilations.creating {
        defaultSourceSet {
          dependencies {
            // Compile against the main compilation's compile classpath and outputs:
            implementation(main.compileDependencyFiles + main.output.classesDirs)
          }
        }
      }
    }
  }

  sourceSets {
    val jvmMain by getting {
      dependencies {
        implementation(kotlin("stdlib-common"))
        implementation(project(":flatbuffers-kotlin"))
        implementation(libs.kotlinx.benchmark.runtime)
        // json serializers
        implementation(libs.moshi.kotlin)
        implementation(libs.gson)
      }
      kotlin.srcDir("src/jvmMain/generated/kotlin/")
      kotlin.srcDir("src/jvmMain/generated/java/")
      kotlin.srcDir("../../java/src/main/java")
    }
  }
}

// This task download all JSON files used for benchmarking
tasks.register<de.undercouch.gradle.tasks.download.Download>("downloadMultipleFiles") {
  // We are downloading json benchmark samples from serdes-rs project.
  // see: https://github.com/serde-rs/json-benchmark/blob/master/data
  val baseUrl = "https://github.com/serde-rs/json-benchmark/raw/master/data/"
  src(listOf("$baseUrl/canada.json", "$baseUrl/twitter.json", "$baseUrl/citm_catalog.json"))
  dest(File("${project.projectDir.absolutePath}/src/jvmMain/resources"))
  overwrite(false)
}

abstract class GenerateFBTestClasses : DefaultTask() {
  @get:InputFiles
  abstract val inputFiles: ConfigurableFileCollection

  @get:Input
  abstract val includeFolder: Property<String>

  @get:Input
  abstract val outputFolder: Property<String>

  @get:Input
  abstract val variants: ListProperty<String>

  @Inject
  protected open fun getExecActionFactory(): org.gradle.process.internal.ExecActionFactory? {
    throw UnsupportedOperationException()
  }

  init {
    includeFolder.set("")
  }

  @TaskAction
  fun compile() {
    val execAction = getExecActionFactory()!!.newExecAction()
    val sources = inputFiles.asPath.split(":")
    val langs = variants.get().map { "--$it" }
    val args = mutableListOf("flatc","-o", outputFolder.get(), *langs.toTypedArray())
    if (includeFolder.get().isNotEmpty()) {
      args.add("-I")
      args.add(includeFolder.get())
    }
    args.addAll(sources)
    println(args)
    execAction.commandLine = args
    print(execAction.execute())
  }
}

// Use the default greeting
tasks.register<GenerateFBTestClasses>("generateFBTestClassesKt") {
  inputFiles.setFrom("$projectDir/monster_test_kotlin.fbs")
  includeFolder.set("$rootDir/../tests/include_test")
  outputFolder.set("${projectDir}/src/jvmMain/generated/kotlin/")
  variants.addAll("kotlin-kmp")
}

tasks.register<GenerateFBTestClasses>("generateFBTestClassesJava") {
  inputFiles.setFrom("$projectDir/monster_test_java.fbs")
  includeFolder.set("$rootDir/../tests/include_test")
  outputFolder.set("${projectDir}/src/jvmMain/generated/java/")
  variants.addAll("kotlin")
}

project.tasks.forEach {
  if (it.name.contains("compileKotlin")) {
    it.dependsOn("generateFBTestClassesKt")
    it.dependsOn("generateFBTestClassesJava")
  }
}
