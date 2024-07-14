<script lang="ts">
  import IconLoading from '~icons/mdi/loading';

  export let isLoading = false;
  export let isDisabled = false;

  let className = '';
  export { className as class };
</script>

<button class={`${className ?? ''}`} {...$$restProps} disabled={isDisabled || isLoading}>
  <span class="flex gap-2 justify-center items-center">
    {#if $$slots.leftIcon}
      <slot name="leftIcon" />
    {/if}

    <slot />

    {#if $$slots.rightIcon && !isLoading}
      <slot name="rightIcon" />
    {/if}

    {#if isLoading}
      <IconLoading class="animate-spin" />
    {/if}
  </span>
</button>
