  <script lang="ts">
    export let label: string;
    export let value: number;
  </script>
  
  <div class="uppercase px-6 py-5 border-r flex flex-col divide-y">
    <h3 class="text-sm pb-2">{label}</h3>
    <div class="text-xl pt-2 font-bold">{value}</div>
  </div>
