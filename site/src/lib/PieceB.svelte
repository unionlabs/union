<script lang="ts">
  // Clockwise, starting with north east
  type Rotation = 1 | 2 | 3 | 4;
  export let rotation: Rotation = 1; 
</script>

<div class={`curve-path bg-primary rotation-${rotation}`}/>

<style lang="postcss">
  .curve-path {
    clip-path: path('M 0 32 C 17.6731 32 32 17.6731 32 0 L 16 0 C 16 8.83655 8.83655 16 0 16 L 0 32 Z');
    width: var(--gh);
    height: var(--gh);
  }

  @media (min-width: theme(screens.md))  {
    .curve-path {
      clip-path: path('M 0 64 C 35.3462 64 64 35.3462 64 0 L 32 0 C 32 17.6731 17.6731 32 0 32 L 0 64 Z');
    }
  }

  .rotation-1 { transform: rotate(-90deg); }
  .rotation-2 { transform: rotate(0deg); }
  .rotation-3 { transform: rotate(90deg); }
  .rotation-4 { transform: rotate(180deg); }
</style>

<!--
M 0 64 
C 35.3462 64 64 35.3462 64 0 
L 32 0 
C 32 17.6731 17.6731 32 0 32
L 0 64
Z

https://svg-path-visualizer.netlify.app/#M%200%2064%20%0AC%2035.3462%2064%2064%2035.3462%2064%200%20%0AL%2032%200%20%0AC%2032%2017.6731%2017.6731%2032%200%2032%0AL%200%2064%0AZ
!-->

