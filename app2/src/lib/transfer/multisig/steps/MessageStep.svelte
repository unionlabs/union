<script lang="ts">
  type Props = {
    stepIndex: number
    onBack: () => void
    onSubmit: () => void
    step: any
  }

  const { step, onBack, onSubmit }: Props = $props()
</script>