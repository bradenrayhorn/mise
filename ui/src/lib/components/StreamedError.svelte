<script lang="ts">
  import { goto } from '$app/navigation';
  import type { MaybeError } from '$lib/types/error';
  import { onMount } from 'svelte';

  export let error: MaybeError;

  onMount(() => {
    console.error('streamed error: ', error);

    if (error?.status && error?.status >= 300 && error?.status <= 399 && error?.location) {
      goto(error?.location);
    }
  });
</script>

<slot />
