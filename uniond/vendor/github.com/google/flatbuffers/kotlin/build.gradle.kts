import org.gradle.internal.impldep.org.testng.ITestResult.STARTED
import org.jetbrains.kotlin.gradle.dsl.KotlinCommonOptions
import org.jetbrains.kotlin.gradle.tasks.KotlinCompile
import java.nio.charset.StandardCharsets

buildscript {
  repositories {
    gradlePluginPortal()
    google()
    mavenCentral()
  }
  dependencies {
    classpath(libs.plugin.kotlin.gradle)
    classpath(libs.plugin.kotlinx.benchmark)
    classpath(libs.plugin.jmhreport)
    classpath(libs.plugin.download)
  }
}

allprojects {
  repositories {
    google()
    mavenCentral()
  }
}

tasks.withType<org.jetbrains.kotlin.gradle.dsl.KotlinCompile<KotlinCommonOptions>>().configureEach {
  kotlinOptions {
    freeCompilerArgs += "-progressive" // https://kotlinlang.org/docs/whatsnew13.html#progressive-mode
  }
}

tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinJvmCompile>().configureEach {
  kotlinOptions {
    jvmTarget = JavaVersion.VERSION_1_8.toString()
    freeCompilerArgs += "-Xjvm-default=all"
  }
}

tasks.withType<JavaCompile> {
  options.encoding = StandardCharsets.UTF_8.toString()
  sourceCompatibility = JavaVersion.VERSION_1_8.toString()
  targetCompatibility = JavaVersion.VERSION_1_8.toString()
}
