package com.flatbuffers.app

import android.annotation.SuppressLint
import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import android.widget.TextView
import com.fbs.app.Animal
import com.google.flatbuffers.FlatBufferBuilder
import java.nio.ByteBuffer

@ExperimentalUnsignedTypes
class MainActivity : AppCompatActivity() {

  @SuppressLint("SetTextI18n")
  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)
    setContentView(R.layout.activity_main)

    val tiger = Animal.getRootAsAnimal(ByteBuffer.wrap(createAnimalFromJNI()))
    findViewById<TextView>(R.id.tv_animal_one).text = animalInfo(tiger)

    findViewById<TextView>(R.id.tv_animal_two).text = animalInfo(createAnimalFromKotlin())
  }

  // This function is a sample of communicating FlatBuffers between JNI (native C++) and Java.
  // Implementation can be found on animals.cpp file.
  private external fun createAnimalFromJNI(): ByteArray

  // Create a "Cow" Animal flatbuffers from Kotlin
  private fun createAnimalFromKotlin():Animal {
    val fb = FlatBufferBuilder(100)
    val cowOffset = Animal.createAnimal(
      builder = fb,
      nameOffset = fb.createString("Cow"),
      soundOffset = fb.createString("Moo"),
      weight = 720u
    )
    fb.finish(cowOffset)
    return Animal.getRootAsAnimal(fb.dataBuffer())
  }

  private fun animalInfo(animal: Animal): String =
    "The ${animal.name} sound is ${animal.sound} and it weights ${animal.weight}kg."

  companion object {
    // Used to load the 'native-lib' library on application startup.
    init {
      System.loadLibrary("native-lib")
    }
  }
}
