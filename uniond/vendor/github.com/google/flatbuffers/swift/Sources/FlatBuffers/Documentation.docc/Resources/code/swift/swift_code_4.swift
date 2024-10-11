import FlatBuffers
import Foundation

func run() {
  // create a `FlatBufferBuilder`, which will be used to serialize objects
  let builder = FlatBufferBuilder(initialSize: 1024)

  let weapon1Name = builder.create(string: "Sword")
  let weapon2Name = builder.create(string: "Axe")
}
