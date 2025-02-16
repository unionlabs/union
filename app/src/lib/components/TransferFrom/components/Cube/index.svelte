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
let currentFace: CubeFaces = "intentFace"
let targetFace: CubeFaces = currentFace
let isRotating = false

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

function handleTransitionEnd(e: TransitionEvent) {
  if (e.propertyName === "transform") {
    isRotating = false
    currentFace = targetFace
  }
}

function rotateTo(face: CubeFaces) {
  targetFace = face
  isRotating = true

  // Update visibility state immediately
  if (face === "sourceFace") {
    currentVisibleFace = "source"
  } else if (face === "destinationFace") {
    currentVisibleFace = "destination"
  }

  // Delay the rotation by 100ms
  setTimeout(() => {
    const targetRotation = facePositions[face]

    // Calculate the new Y rotation
    const newY = findShortestRotation(currentRotation.y, targetRotation)
    currentRotation = { x: 0, y: newY }
  }, 100)
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

$: height = width * 1.6
$: translateZ = width / 2
</script>

<div class="h-screen w-full flex items-center justify-center">
  {#if isRotating}
    <div
            class="relative transform-style-preserve-3d transition-transform duration-500 h-full"
            style={`width: ${width}px; height: ${height}px; transform: rotateX(${currentRotation.x}deg) rotateY(${currentRotation.y}deg)`}
            on:transitionend={handleTransitionEnd}
    >
      <FaceWrapper {width} {height} {translateZ} rotateY={"0deg"}>
        <slot name="intent" {rotateTo}/>
      </FaceWrapper>

      {#if currentVisibleFace === 'source'}
        <FaceWrapper {width} {height} {translateZ} rotateY={"90deg"}>
          <slot name="source" {rotateTo}/>
        </FaceWrapper>
      {:else if currentVisibleFace === 'destination'}
        <FaceWrapper {width} {height} {translateZ} rotateY={"90deg"}>
          <slot name="destination" {rotateTo}/>
        </FaceWrapper>
      {/if}

      <FaceWrapper {width} {height} {translateZ} rotateY={"270deg"}>
        <slot name="assets" {rotateTo}/>
      </FaceWrapper>

      <FaceWrapper {width} {height} {translateZ} rotateY={"180deg"}>
        <slot name="transfer" {rotateTo}/>
      </FaceWrapper>
    </div>
  {:else}
    <div
            class="relative   h-full"
            style={`width: ${width}px; height: ${height}px;`}
            on:transitionend={handleTransitionEnd}
    >
      {#if currentFace === "intentFace"}
        <div class="bg-muted border-2 h-full w-full" style={`width: ${width}px; height: ${height}px"`}>
          <slot name="intent" {rotateTo}/>
        </div>
      {:else if currentFace === "destinationFace" || currentFace === "sourceFace"}
        {#if currentVisibleFace === 'source'}
          <div class="bg-muted border-2 h-full w-full" style={`width: ${width}px; height: ${height}px;`}>
            <slot name="source" {rotateTo}/>
          </div>
        {:else if currentVisibleFace === 'destination'}
          <div class="bg-muted border-2 h-full w-full" style={`width: ${width}px; height: ${height}px;`}>
            <slot name="destination" {rotateTo}/>
          </div>
        {/if}
      {:else if currentFace === 'assetsFace'}
        <div class="bg-muted border-2 h-full w-full" style={`width: ${width}px; height: ${height}px;`}>
          <slot name="assets" {rotateTo}/>
        </div>
      {:else if currentFace === "verifyFace"}
        <div class="bg-muted border-2 h-full w-full" style={`width: ${width}px; height: ${height}px;`}>
          <slot name="transfer" {rotateTo}/>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
.perspective {
  perspective: 2000px;
}

  .transform-style-preserve-3d {
    transform-style: preserve-3d;
  }
</style>
