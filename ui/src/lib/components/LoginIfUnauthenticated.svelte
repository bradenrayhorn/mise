<script lang="ts">
  import { useAuth } from '$lib/auth-context';
  import PageReauthenticateState from './page-states/PageReauthenticateState.svelte';

  const auth = useAuth();
</script>

{#if $auth.unauthenticated}
  <PageReauthenticateState />
{:else}
  <slot />
{/if}
