<script lang="ts">
  import { queryKeys } from '$lib/api/query-keys';
  import { createQuery } from '@tanstack/svelte-query';
  import SettingsPage from './SettingsPage.svelte';
  import { getSelf } from '$lib/api/load/user';
  import PageLoadingState from '$lib/components/page-states/PageLoadingState.svelte';
  import PageErrorState from '$lib/components/page-states/PageErrorState.svelte';

  $: query = createQuery<string>({
    queryKey: [queryKeys.user.self],
    queryFn: () => getSelf({ fetch }),
    refetchOnMount: true,
    staleTime: 0,
    gcTime: 0,
  });

  const backURL = `/recipes?${localStorage.getItem('last-recipes-query')}`;
</script>

{#if $query.isPending}
  <PageLoadingState />
{:else if $query.isError}
  <PageErrorState error={$query.error} />
{:else}
  <SettingsPage {backURL} />
{/if}
