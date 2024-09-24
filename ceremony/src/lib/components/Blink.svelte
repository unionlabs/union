<script lang="ts">
type Props = {
  loading?: boolean
  sleep?: boolean
  dead?: boolean
  love?: boolean
}

let { loading = false, sleep = false, dead = false, love = false }: Props = $props()

let eye = $state(dead ? "×" : sleep ? "-" : love ? "♡" : "0")
let blinkInterval: number | NodeJS.Timeout

function blinkEye() {
  if (!(sleep || love || dead)) {
    eye = "-"
    setTimeout(() => {
      eye = "0"
    }, 100)
  }
}

function startRandomBlinking() {
  blinkInterval = setInterval(() => {
    if (!(sleep || love || dead) && Math.random() < 0.05) {
      blinkEye()
    }
  }, 200)
}

$effect(() => {
  if (dead) {
    eye = "×"
    clearInterval(blinkInterval)
  } else if (sleep) {
    eye = "-"
    clearInterval(blinkInterval)
  } else if (love) {
    eye = "♡"
    clearInterval(blinkInterval)
  } else {
    eye = "0"
    startRandomBlinking()
  }

  return () => {
    clearInterval(blinkInterval)
  }
})
</script>

<span class="!text-union-accent-500">
  {eye}<span class:wobble={loading && !sleep && !dead}><span>_</span><span>_</span><span>_</span><span>_</span><span>_</span><span>_</span><span>_</span></span>{eye}
</span>

<style>
    .wobble {
        display: inline-block;
    }

    .wobble span {
        display: inline-block;
        animation: wobble 1s infinite ease-in-out;
    }

    @keyframes wobble {
        0% {
            transform: translateY(0);
        }
        25% {
            transform: translateY(-3px);
        }
        50% {
            transform: translateY(0);
        }
        75% {
            transform: translateY(3px);
        }
        100% {
            transform: translateY(0);
        }
    }

    .wobble span:nth-child(1) { animation-delay: 0s; }
    .wobble span:nth-child(2) { animation-delay: 0.1s; }
    .wobble span:nth-child(3) { animation-delay: 0.2s; }
    .wobble span:nth-child(4) { animation-delay: 0.3s; }
    .wobble span:nth-child(5) { animation-delay: 0.4s; }
    .wobble span:nth-child(6) { animation-delay: 0.5s; }
    .wobble span:nth-child(7) { animation-delay: 0.6s; }
</style>