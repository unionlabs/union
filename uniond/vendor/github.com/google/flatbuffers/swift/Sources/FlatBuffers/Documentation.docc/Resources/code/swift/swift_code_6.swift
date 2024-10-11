import FlatBuffers
import Foundation

func run() {
  // create a `FlatBufferBuilder`, which will be used to serialize objects
  let builder = FlatBufferBuilder(initialSize: 1024)

  let weapon1Name = builder.create(string: "Sword")
  let weapon2Name = builder.create(string: "Axe")

  // start creating the weapon by calling startWeapon
  let weapon1Start = Weapon.startWeapon(&builder)
  Weapon.add(name: weapon1Name, &builder)
  Weapon.add(damage: 3, &builder)
  // end the object by passing the start point for the weapon 1
  let sword = Weapon.endWeapon(&builder, start: weapon1Start)

  let weapon2Start = Weapon.startWeapon(&builder)
  Weapon.add(name: weapon2Name, &builder)
  Weapon.add(damage: 5, &builder)
  let axe = Weapon.endWeapon(&builder, start: weapon2Start)

  // Create a FlatBuffer `vector` that contains offsets to the sword and axe
  // we created above.
  let weaponsOffset = builder.createVector(ofOffsets: [sword, axe])
}
