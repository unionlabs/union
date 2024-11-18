<script lang="ts">
import clsx from "clsx"

interface Props {
  rows: Array<Array<string>>
  headers: Array<string>
}

let { rows, headers }: Props = $props()
</script>

<div class="flex flex-col justify-center">
  <div class="inline-block align-middle">
    <div class="align-middle border rounded-sm border-neutral-400">
      <table class="divide-y divide-gray-300 tabular-nums">
        <thead class="">
          <tr class="divide-x divide-gray-200">
            {#each headers as header, index}
              <th scope="col" class={clsx([`py-3.5 text-left font-semibold px-4`])}>
                {header}
              </th>
            {/each}
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-200">
          {#each rows as row, index}
            <tr class="">
              {#each row as cell, index}
                <td class="whitespace-nowrap p-4 font-medium">{cell}</td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</div>

<style lang="postcss">
  table {
    width: 100%;
    display: table;
  }
</style>
