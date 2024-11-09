<script lang="ts">
  import PageReauthenticateState from './page-states/PageReauthenticateState.svelte';
  import type { Snippet } from 'svelte';
  import { auth } from '$lib/auth.svelte';

  type Props = {
    children: Snippet;
  };

  let { children }: Props = $props();
</script>

{#if auth.unauthenticated}
  <PageReauthenticateState />
{:else}
  {@render children?.()}
{/if}
