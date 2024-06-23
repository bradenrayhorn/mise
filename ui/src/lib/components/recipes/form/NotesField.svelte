<script lang="ts">
  export let value: string;
  export let errors: string[] | undefined = undefined;
</script>

<label>
  <span>Notes</span>
  <textarea
    class="border border-black"
    bind:value
    aria-invalid={errors ? 'true' : undefined}
    {...$$restProps}
  />
</label>
{#if errors}<span class="invalid">{errors}</span>{/if}
