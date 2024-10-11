@file:OptIn(ExperimentalUnsignedTypes::class)

package com.google.flatbuffers.kotlin.benchmark


import com.google.flatbuffers.kotlin.FlatBufferBuilder
import jmonster.JAllMonsters
import jmonster.JColor
import jmonster.JMonster
import jmonster.JVec3
import monster.AllMonsters
import monster.AllMonsters.Companion.createAllMonsters
import monster.AllMonsters.Companion.createMonstersVector
import monster.Monster
import monster.Monster.Companion.createInventoryVector
import monster.MonsterOffsetArray
import monster.Vec3
import org.openjdk.jmh.annotations.*
import org.openjdk.jmh.infra.Blackhole
import java.util.concurrent.TimeUnit

@State(Scope.Benchmark)
@BenchmarkMode(Mode.AverageTime)
@OutputTimeUnit(TimeUnit.NANOSECONDS)
@Measurement(iterations = 20, time = 1, timeUnit = TimeUnit.NANOSECONDS)
open class FlatbufferBenchmark {

  val repetition = 1000000
  val fbKotlin = FlatBufferBuilder(1024 * repetition)
  val fbDeserializationKotlin = FlatBufferBuilder(1024 * repetition)
  val fbJava = com.google.flatbuffers.FlatBufferBuilder(1024 * repetition)
  val fbDeserializationJava = com.google.flatbuffers.FlatBufferBuilder(1024 * repetition)

  init {
      populateMosterKotlin(fbDeserializationKotlin)
      populateMosterJava(fbDeserializationJava)
  }
  @OptIn(ExperimentalUnsignedTypes::class)
  private fun populateMosterKotlin(fb: FlatBufferBuilder) {
    fb.clear()
    val monsterName = fb.createString("MonsterName");
    val items = ubyteArrayOf(0u, 1u, 2u, 3u, 4u)
    val inv = createInventoryVector(fb, items)
    val monsterOffsets: MonsterOffsetArray = MonsterOffsetArray(repetition) {
      Monster.startMonster(fb)
      Monster.addName(fb, monsterName)
      Monster.addPos(fb, Vec3.createVec3(fb, 1.0f, 2.0f, 3.0f))
      Monster.addHp(fb, 80)
      Monster.addMana(fb, 150)
      Monster.addInventory(fb, inv)
      Monster.addColor(fb, monster.Color.Red)
      Monster.endMonster(fb)
    }
    val monsters = createMonstersVector(fb, monsterOffsets)
    val allMonsters = createAllMonsters(fb, monsters)
    fb.finish(allMonsters)
  }

  @OptIn(ExperimentalUnsignedTypes::class)
  private fun populateMosterJava(fb: com.google.flatbuffers.FlatBufferBuilder){
    fb.clear()
    val monsterName = fb.createString("MonsterName");
    val inv = JMonster.createInventoryVector(fb, ubyteArrayOf(0u, 1u, 2u, 3u, 4u))
    val monsters = JAllMonsters.createMonstersVector(fb, IntArray(repetition) {
      JMonster.startJMonster(fb)
      JMonster.addName(fb, monsterName)
      JMonster.addPos(fb, JVec3.createJVec3(fb, 1.0f, 2.0f, 3.0f))
      JMonster.addHp(fb, 80)
      JMonster.addMana(fb, 150)
      JMonster.addInventory(fb, inv)
      JMonster.addColor(fb, JColor.Red)
      JMonster.endJMonster(fb)
    })
    val allMonsters = JAllMonsters.createJAllMonsters(fb, monsters)
    fb.finish(allMonsters)
  }
  @Benchmark
  fun monstersSerializationKotlin() {
    populateMosterKotlin(fbKotlin)
  }

  @OptIn(ExperimentalUnsignedTypes::class)
  @Benchmark
  fun monstersDeserializationKotlin(hole: Blackhole) {
    val monstersRef = AllMonsters.asRoot(fbDeserializationKotlin.dataBuffer())

    for (i in 0 until monstersRef.monstersLength) {
      val monster = monstersRef.monsters(i)!!
      val pos = monster.pos!!
      hole.consume(monster.name)
      hole.consume(pos.x)
      hole.consume(pos.y)
      hole.consume(pos.z)
      hole.consume(monster.hp)
      hole.consume(monster.mana)
      hole.consume(monster.color)
      hole.consume(monster.inventory(0).toByte())
      hole.consume(monster.inventory(1))
      hole.consume(monster.inventory(2))
      hole.consume(monster.inventory(3))
    }
  }
  @Benchmark
  fun monstersSerializationJava() {
    populateMosterJava(fbJava)
  }

  @Benchmark
  fun monstersDeserializationJava(hole: Blackhole) {
    val monstersRef = JAllMonsters.getRootAsJAllMonsters(fbDeserializationJava.dataBuffer())

    for (i in 0 until monstersRef.monstersLength) {
      val monster = monstersRef.monsters(i)!!
      val pos = monster.pos!!
      hole.consume(monster.name)
      hole.consume(pos.x)
      hole.consume(pos.y)
      hole.consume(pos.z)
      hole.consume(monster.hp)
      hole.consume(monster.mana)
      hole.consume(monster.color)
      hole.consume(monster.inventory(0))
      hole.consume(monster.inventory(1))
      hole.consume(monster.inventory(2))
      hole.consume(monster.inventory(3))
    }
  }

}
