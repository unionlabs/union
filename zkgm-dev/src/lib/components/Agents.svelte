<script>
import { onMount, onDestroy } from "svelte"
import { browser } from "$app/environment"

let containerWidth
let containerRef

let characters = [
  { id: "omar", x: 10, direction: 1, speed: 2 },
  { id: "lukas", x: 300, direction: -1, speed: 3 },
  { id: "ben", x: 600, direction: 1, speed: 1.5 },
  { id: "cor", x: 900, direction: -1, speed: 2.5 }
]

const CHARACTER_WIDTH = 48
const COLLISION_THRESHOLD = 2
const MIN_SPEED = 1
const MAX_SPEED = 4
let animationFrameId

function getRandomSpeed() {
  return MIN_SPEED + Math.random() * (MAX_SPEED - MIN_SPEED)
}

function handleResize() {
  if (containerRef) {
    containerWidth = containerRef.offsetWidth
  }
}

function updatePositions() {
  const newPositions = characters.map(char => {
    const newX = char.x + char.speed * char.direction
    return { ...char, newX }
  })

  characters = newPositions.map((char, index) => {
    let finalX = char.newX
    let newDirection = char.direction
    let newSpeed = char.speed

    if (finalX <= 0) {
      newDirection = 1
      finalX = 0
      newSpeed = getRandomSpeed()
    } else if (finalX >= containerWidth - CHARACTER_WIDTH) {
      newDirection = -1
      finalX = containerWidth - CHARACTER_WIDTH
      newSpeed = getRandomSpeed()
    }

    for (let i = 0; i < newPositions.length; i++) {
      if (i !== index) {
        const other = newPositions[i]

        const distance =
          char.direction > 0
            ? other.newX - (char.newX + CHARACTER_WIDTH)
            : char.newX - (other.newX + CHARACTER_WIDTH)

        const isApproaching =
          (char.direction > 0 && other.direction < 0) || (char.direction < 0 && other.direction > 0)

        if (isApproaching && Math.abs(distance) <= COLLISION_THRESHOLD) {
          finalX = char.x
          newDirection *= -1
          newSpeed = getRandomSpeed()
          break
        }
      }
    }

    return {
      ...char,
      x: finalX,
      direction: newDirection,
      speed: newSpeed
    }
  })

  animationFrameId = browser ? requestAnimationFrame(updatePositions) : null
}

onMount(() => {
  if (browser) {
    handleResize()
    window.addEventListener("resize", handleResize)
    animationFrameId = requestAnimationFrame(updatePositions)
  }
})

onDestroy(() => {
  if (browser) {
    window.removeEventListener("resize", handleResize)
    if (animationFrameId) {
      cancelAnimationFrame(animationFrameId)
    }
  }
})
</script>

<div
        bind:this={containerRef}
        class="relative w-full h-full bg-transparent overflow-hidden"
>
  {#each characters as char (char.id)}
    <div
            class="absolute bottom-0 transition-transform duration-100 ease-linear"
            style="transform: translateX({char.x}px) scaleX({char.direction > 0 ? 1 : -1})"
    >
      <img
              src="/agents/{char.id}.png"
              alt="Agent {char.id}"
              class="h-16 w-16 flex-shrink-0"
      />
    </div>
  {/each}
</div>