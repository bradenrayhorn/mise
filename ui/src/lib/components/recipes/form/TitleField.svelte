<script lang="ts">
  export let value: string;
  export let errors: string[] | undefined = undefined;
</script>

<label>
  <span>Title</span>
  <input
    class="border border-black"
    type="text"
    bind:value
    aria-invalid={errors ? 'true' : undefined}
    {...$$restProps}
  />
</label>
{#if errors}<span class="invalid">{errors}</span>{/if}
