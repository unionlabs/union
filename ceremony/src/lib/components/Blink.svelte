<script lang="ts">
import H1 from "$lib/components/typography/H1.svelte"

let eye = $state("0")
let blinkInterval: number | NodeJS.Timeout

function blinkEye() {
  eye = "-"
  setTimeout(() => {
    eye = "0"
  }, 100)
}

function startRandomBlinking() {
  blinkInterval = setInterval(() => {
    if (Math.random() < 0.05) {
      blinkEye()
    }
  }, 200)
}

$effect(() => {
  startRandomBlinking()

  return () => {
    clearInterval(blinkInterval)
  }
})
</script>

<H1 class="!text-union-accent-500">{eye}_______{eye}</H1>