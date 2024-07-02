  <script lang="ts">
    export let label: string;
    export let value: number | string;
  </script>
  
  <div class="uppercase w-fit flex px-6 py-5 border-r">
    <div class="flex flex-col divide-y">
      <h3 class="text-sm pb-2">{label}</h3>
      <div class="text-xl pt-2 font-bold">{value}</div>
    </div>
    <slot />
  </div>
