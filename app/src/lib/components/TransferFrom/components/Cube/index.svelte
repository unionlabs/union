<script lang="ts">
import FaceWrapper from "$lib/components/TransferFrom/components/Cube/FaceWrapper.svelte"
import { deviceWidth } from "$lib/utilities/device.ts"

type CubeFaces =
  | "intentFace"
  | "chainsFace"
  | "verifyFace"
  | "assetsFace"
  | "sourceFace"
  | "destinationFace"

let currentRotation = { x: 0, y: 0 }

const facePositions = {
  intentFace: 0,
  chainsFace: -90,
  verifyFace: -180,
  assetsFace: -270,
  sourceFace: -90,
  destinationFace: -90
} as const

let currentVisibleFace: "source" | "destination" = "source"
$: currentVisibleFace = "source"

function findShortestRotation(current: number, target: number): number {
  const revolution = Math.floor(current / 360) * 360
  const normalizedTarget = target + revolution

  let diff = normalizedTarget - current
  if (Math.abs(diff) > 180) {
    diff = diff > 0 ? diff - 360 : diff + 360
  }
  return current + diff
}

function rotateTo(face: CubeFaces) {
  console.log("rotate to: ", face)
  const targetRotation = facePositions[face]

  // Calculate the new Y rotation
  const newY = findShortestRotation(currentRotation.y, targetRotation)
  currentRotation = { x: 0, y: newY }

  // Update visibility state
  if (face === "sourceFace") {
    currentVisibleFace = "source"
  } else if (face === "destinationFace") {
    currentVisibleFace = "destination"
  }
}

//If we want to be specific we can set each w
$: width =
  $deviceWidth >= 1536
    ? 400
    : // 2xl breakpoint
      $deviceWidth >= 1280
      ? 400
      : // xl breakpoint
        $deviceWidth >= 1024
        ? 400
        : // lg breakpoint
          $deviceWidth >= 768
          ? 400
          : // md breakpoint
            $deviceWidth >= 640
            ? 400
            : // sm breakpoint
              300 // Default for smaller screens

$: height = width * 1.55
$: translateZ = width / 2
</script>

<div class="h-screen w-full flex items-center justify-center perspective-[2000px]">
  <div
          class="relative transform-style-preserve-3d transition-transform duration-500 h-full"
          style={`width: ${width}px; height: ${height}px; transform: rotateX(${currentRotation.x}deg) rotateY(${currentRotation.y}deg)`}
  >
    <FaceWrapper {width} {height} {translateZ} visible rotateY={"0deg"}>
      <slot name="intent" {rotateTo}/>
    </FaceWrapper>

    <!--Source and destination is on the same degree, we just hide one depending on clicked intent.-->
    <!--By doing this we can "layer" faces and reuse the rotation-->
    <FaceWrapper {width} {height} {translateZ} visible={currentVisibleFace === 'source'} rotateY={"90deg"}>
      <slot name="source" {rotateTo}/>
    </FaceWrapper>

    <FaceWrapper {width} {height} {translateZ} visible={currentVisibleFace === 'destination'} rotateY={"90deg"}>
      <slot name="destination" {rotateTo}/>
    </FaceWrapper>

    <FaceWrapper {width} {height} {translateZ} visible rotateY={"270deg"}>
      <slot name="assets" {rotateTo}/>
    </FaceWrapper>

    <FaceWrapper {width} {height} {translateZ} visible rotateY={"180deg"}>
      <slot name="transfer" {rotateTo}/>
    </FaceWrapper>
  </div>
</div>

<style>
  .perspective-\[2000px\] {
    perspective: 2000px;
  }

  .transform-style-preserve-3d {
    transform-style: preserve-3d;
  }
</style>
