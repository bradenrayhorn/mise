<script lang="ts">
  import { auth } from '$lib/auth.svelte';
  import type { MaybeError } from '$lib/types/error';
  import IconChef from '~icons/mdi/chef-hat';

  interface Props {
    error: MaybeError;
  }

  let { error }: Props = $props();

  if (error?.status === 401) {
    auth.unauthenticated = true;
  }
</script>

<div class="bg-base-500 fixed top-0 left-0 bottom-0 right-0 flex justify-center items-center">
  <div class="flex flex-col items-center gap-4">
    <IconChef class="text-7xl md:text-8xl text-fg-default" />

    <span>There was an error loading the page.</span>

    <button
      class="btn-link"
      onclick={() => {
        window.location.reload();
      }}>Click here to try again.</button
    >
  </div>
</div>
