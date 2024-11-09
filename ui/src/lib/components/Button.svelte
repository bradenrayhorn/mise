<script lang="ts">
  import IconLoading from '~icons/mdi/loading';
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';

  type Props = {
    isLoading?: boolean;
    isDisabled?: boolean;
    class?: string;
    leftIcon?: Snippet;
    children?: Snippet;
    rightIcon?: Snippet;
  } & HTMLButtonAttributes;

  let {
    isLoading = false,
    isDisabled = false,
    class: className = '',
    leftIcon,
    children,
    rightIcon,
    ...rest
  }: Props = $props();

  const children_render = $derived(children);
</script>

<button class={`${className ?? ''}`} {...rest} disabled={isDisabled || isLoading}>
  <span class="flex gap-2 justify-center items-center">
    {#if leftIcon}
      {@render leftIcon?.()}
    {/if}

    {@render children_render?.()}

    {#if rightIcon && !isLoading}
      {@render rightIcon?.()}
    {/if}

    {#if isLoading}
      <IconLoading class="animate-spin" />
    {/if}
  </span>
</button>
