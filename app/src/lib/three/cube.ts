import * as THREE from "three"

const darkModeColor = 0xffffff
const lightModeColor = 0x000000

export function createCube(cubeWidth: number, strokeWidth: number): THREE.Group {
  const boxGeometry = new THREE.BoxGeometry(cubeWidth, cubeWidth, cubeWidth)
  const edgesGeometry = new THREE.EdgesGeometry(boxGeometry)

  function checkIfDark(): boolean {
    return window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches
  }

  const createTubeLine = (points, thickness) => {
    const path = new THREE.CatmullRomCurve3(points)
    const geometry = new THREE.TubeGeometry(path, 20, thickness, 8, false)
    const material = new THREE.MeshBasicMaterial(
      checkIfDark() ? { color: darkModeColor } : { color: lightModeColor }
    )
    const tube = new THREE.Mesh(geometry, material)
    return tube
  }

  const edgesGroup = new THREE.Group() // Create a group to hold all edges

  for (let i = 0; i < edgesGeometry.attributes.position.count; i += 2) {
    const start = new THREE.Vector3(
      edgesGeometry.attributes.position.getX(i),
      edgesGeometry.attributes.position.getY(i),
      edgesGeometry.attributes.position.getZ(i)
    )
    const end = new THREE.Vector3(
      edgesGeometry.attributes.position.getX(i + 1),
      edgesGeometry.attributes.position.getY(i + 1),
      edgesGeometry.attributes.position.getZ(i + 1)
    )
    const line = createTubeLine([start, end], strokeWidth)
    edgesGroup.add(line) // Add each edge to the group
  }
  return edgesGroup
}
