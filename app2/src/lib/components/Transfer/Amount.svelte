<script lang="ts">
  import Input from "$lib/components/ui/Input.svelte"
  import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"
  import { Option } from "effect"
</script>
<Input id="amount"
       label="amount"
       type="text"
       required={true}
       disabled={!transfer.raw.asset}
       autocorrect="off"
       placeholder="0.00"
       spellcheck="false"
       autocomplete="off"
       inputmode="decimal"
       data-field="amount"
       autocapitalize="none"
       pattern="^[0-9]*[.]?[0-9]*$"
       value={transfer.raw.amount}
       oninput={(event) => {
                const input = event.currentTarget;
                const value = input.value;
                const maxDecimals = Option.isSome(transfer.baseToken)
                  ? (transfer.baseToken.value.representations[0]?.decimals ?? 0)
                  : 0;

                if (value === '' || (/^\d*\.?\d*$/.test(value) &&
                  (value.includes('.')
                    ? value.split('.')[1].length <= maxDecimals
                    : true)
                )) {
                  transfer.raw.updateField('amount', event);
                } else {
                  input.value = transfer.raw.amount;
                }
              }}
       class="text-center"
/>