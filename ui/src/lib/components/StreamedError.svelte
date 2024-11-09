<script lang="ts">
  import { auth } from '$lib/auth.svelte';
  import type { MaybeError } from '$lib/types/error';
  import { onMount, type Snippet } from 'svelte';

  type Props = {
    error: MaybeError;
    children?: Snippet;
  };

  const { error, children }: Props = $props();

  onMount(() => {
    console.error('streamed error: ', error);

    if (error?.status === 401) {
      auth.unauthenticated = true;
    }
  });
</script>

{@render children?.()}
