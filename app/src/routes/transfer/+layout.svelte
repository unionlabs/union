<svelte:head>
  <title>Union | Send</title>
</svelte:head>

<slot />

<style global lang="postcss">
  .cube-scene {
    @apply absolute -my-6 py-6 z-20;
    top: calc(50% - (var(--height) / 2));
    --width: calc(min(500px, (100dvw - 32px)));
    --height: calc(min(740px, (100dvh - 144px)));
    --depth: 80px;
    --speed: 2s;
    width: var(--width);
    height: var(--height);
    perspective: 1000px;
  }

  .cube {
    @apply relative;
    width: var(--width);
    height: var(--height);
    transform-style: preserve-3d;
    transition: transform var(--speed);
    transform: translateZ(calc(var(--depth) * -0.5)) rotateY(0deg);
  }

  .cube--flipped {
    transform: translateZ(calc(var(--depth) * -0.5)) rotateY(180deg);
  }

  .cube-front,
  .cube-back {
    @apply absolute overflow-y-auto overflow-x-hidden;

    width: var(--width);
    height: var(--height);
  }

  .cube-left {
    @apply absolute bg-card border;
    width: var(--height);
    height: var(--depth);
    top: calc((var(--height) / 2) - (var(--depth) / 2));
    right: calc((var(--width) / 2) - (var(--height) / 2));
    transform: rotateZ(90deg) translateY(calc(var(--width) * 0.5))
      rotateX(-90deg);
  }

  .cube-right {
    @apply absolute bg-card border;
    width: var(--height);
    height: var(--depth);
    top: calc((var(--height) / 2) - (var(--depth) / 2));
    left: calc((var(--width) / 2) - (var(--height) / 2));
    transform: rotateZ(-90deg) translateY(calc(var(--width) * 0.5))
      rotateX(-90deg);
  }

  .cube-front {
    transform: translateZ(calc(var(--depth) * 0.5));
  }

  .cube-back {
    transform: translateZ(calc(var(--depth) * -0.5)) rotateY(180deg);
  }

  .no-transition {
    @apply overflow-y-auto overflow-x-hidden;
    width: var(--width);
    height: var(--height);
    transition: none !important;
  }
</style>
