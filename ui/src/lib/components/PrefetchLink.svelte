<script lang="ts">
  import { goto } from '$app/navigation';
  import { navigating } from '$lib/navigating.svelte';
  import type { Snippet } from 'svelte';
  import type { HTMLAnchorAttributes } from 'svelte/elements';

  type Props = {
    href: string;
    prefetch: () => Promise<void>;
    children?: Snippet<[{ isLoading: boolean }]>;
  } & HTMLAnchorAttributes;

  let { href, prefetch, children, ...rest }: Props = $props();

  const isLoading = $derived(navigating.to === href);

  async function onClick(e: MouseEvent) {
    e.preventDefault();

    navigating.to = href;

    try {
      await prefetch();
    } catch (error: unknown) {
      // ignore any prefetch errors and trigger primary page loading state
      console.error('prefetch error: ', error);
    }

    // only navigate if another navigation hasn't triggered
    if (navigating.to === href) {
      await goto(href);
      navigating.to = undefined;
    }
  }
</script>

<a {href} onclick={onClick} data-loading={isLoading ? true : undefined} {...rest}
  >{@render children?.({ isLoading })}</a
>
