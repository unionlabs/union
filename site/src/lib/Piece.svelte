<script lang="ts">
  import PieceA from "./PieceA.svelte";
  import PieceB from "./PieceB.svelte";

  // todo: don't copy paste type
  type Rotation = 1 | 2 | 3 | 4;

  // NOTE: does not affect piece of kind A
  export let rotation: Rotation = 1; 

  export let kind:  "A" | "B";

  export let x: number;
  export let y: number;

  const style = (() => `grid-column-start: ${x}; grid-row-start: ${y};`)();
  export let origin: "north" | "east" | "south" | "west";

</script>


<div {style} class={`mover ${origin}`}>
  {#if kind === "A"}
    <PieceA />
  {:else if kind === "B" }   
    <PieceB {rotation} />
  {/if}
</div>

<style>
  @keyframes north {
    0% {
      transform: translateY(-200%) scale(0);
    }
    30% {
      transform: translateY(-200%);
    }
    50% {
      transform: translateY(-200%);
    }
    100% {
      transform: translateY(0);
    }
  }

  @keyframes east {
    0% {
      transform: translateX(200%) scale(0);
    }
    30% {
      transform: translateX(200%);
    }
    50% {
      transform: translateX(200%);
    }
    100% {
      transform: translateX(0);
    }
  }

  @keyframes south {
    0% {
      transform: translateY(200%) scale(0);
    }
    30% {
      transform: translateY(200%);
    }
    50% {
      transform: translateY(200%);
    }
    100% {
      transform: translateY(0);
    }
  }

  @keyframes west {
    0% {
      transform: translateX(-200%) scale(0);
    }
    30% {
      transform: translateX(-200%);
    }
    50% {
      transform: translateX(-200%);
    }
    100% {
      transform: translateX(0);
    }
  }

  .mover {

    animation-duration: var(--duration);
    animation-delay: var(--delay);
    animation-timing-function: ease-in-out;
    animation-fill-mode: both;

    /* 
    Both of these properties _should_ not affect rendering,
    but are required in order to render correctly on FireFox 
    */
    transform: translateX(0);
    backface-visibility: hidden;
  }

  .north {
    animation-name: north;
  }

  .east {
    animation-name: east;
  }

  .south {
    animation-name: south;
  }

  .west {
    animation-name: west;
  }

</style>
