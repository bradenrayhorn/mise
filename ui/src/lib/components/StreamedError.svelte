<script lang="ts">
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';

  export let error: any;

  onMount(() => {
    console.error('streamed error: ', error);

    if (error?.status >= 300 && error?.status <= 399 && error?.location) {
      goto(error?.location);
    }
  });
</script>

<slot />
