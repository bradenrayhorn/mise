<script lang="ts">
  import { useAuth } from '$lib/auth-context';
  import type { MaybeError } from '$lib/types/error';
  import { onMount } from 'svelte';

  export let error: MaybeError;

  const auth = useAuth();

  onMount(() => {
    console.error('streamed error: ', error);

    if (error?.status === 401) {
      $auth.unauthenticated = true;
    }
  });
</script>

<slot />
