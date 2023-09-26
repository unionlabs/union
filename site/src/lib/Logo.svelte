<script lang="ts">
  import Piece from '$lib/Piece.svelte';
  const elements = Array.from({ length: 175 }, () => ({}));
</script>

<div class="logo-and-grid-container block relative" >
  <div class="pointer-events-none my-grid overflow-hidden text-xs border-sub border-[1px] md:border-2 absolute -z-50">
    {#each elements as _, i}
      <div class="box-border border-sub border-[1px] md:border-2 grid-box"/>
    {/each}
  </div>
  <div class="pointer-events-none my-grid text-xs box-border m-[1px] md:m-[2px] z-10 absolute logo-container">
    <!-- u !-->
    <Piece kind="A" x={3} y={2} origin="west" --delay="0.2s"/>
    <Piece kind="B" x={3} y={3} origin="west" --delay="0.4s" rotation={3}/>
    <Piece kind="B" x={5} y={3} origin="south" --delay="0.6s" rotation={2}/>
    <Piece kind="A" x={6} y={2} origin="north" --delay="0.8s"/>

    <!-- n !-->
    <Piece kind="A" x={8} y={3} origin="south" --delay="1.2s"/>
    <Piece kind="B" x={10} y={2} origin="north" --delay="1.4s" rotation={1}/>
    <Piece kind="B" x={8} y={2} origin="north" --delay="1.6s" rotation={4}/>
    <Piece kind="A" x={11} y={3} origin="south" --delay="1.8s"/>

  
    <!-- i !-->
    <Piece kind="A" x={13} y={2} origin="north" --delay="2s"/>
    <Piece kind="A" x={13} y={3} origin="south" --delay="2.2s"/>

    <!-- o !-->
    <Piece kind="B" x={17} y={2} origin="north" --delay="2.4s" rotation={1}/>
    <Piece kind="B" x={17} y={3} origin="south" --delay="2.6s" rotation={2}/>
    <Piece kind="B" x={15} y={3} origin="south" --delay="2.8s" rotation={3}/>
    <Piece kind="B" x={15} y={2} origin="north" --delay="3s" rotation={4}/>

    <!-- n !-->
    <Piece kind="A" x={20} y={3} origin="south" --delay="3.2s"/>
    <Piece kind="B" x={22} y={2} origin="east" --delay="3.4s" rotation={1}/>
    <Piece kind="B" x={20} y={2} origin="north" --delay="3.6s" rotation={4}/>
    <Piece kind="A" x={23} y={3} origin="south" --delay="3.8s"/>
  </div>
</div>

<style lang="postcss">
.logo-and-grid-container {
  --gh: 32px;
  --gw: 16px;
  width: calc((var(--gw) * 25) + 2px);
  height: calc((var(--gh) * 3) + 2px);
}

@media (min-width: theme(screens.md))  {
  .logo-and-grid-container {
    --gh: 64px;
    --gw: 32px;
  }
}

.grid-box {
  height: var(--gh);
  width: var(--gw);
}

.my-grid {
  display: grid;
  grid-template: repeat(3, var(--gh)) / repeat(25, var(--gw));
  height: calc((var(--gh) * 7) + 2px);
}

@media (min-width: theme(screens.md))  {
  .my-grid {
    height: calc((var(--gh) * 6) + 4px);
  }
}



.logo-container {
    animation-duration: 0.5s;
    animation-delay: 5.3s;
    animation-name: slideToPos;
    animation-timing-function: ease-in-out;
    animation-fill-mode: both;
}

@keyframes slideToPos {
  0% {
    transform: translateY(var(--gh));
  }
  100% {
    transform: translateY(0);
  }
}

</style>
