<script lang="ts">
  import { getTags } from '$lib/api/load/tag';
  import { queryKeys } from '$lib/api/query-keys';
  import PageErrorState from '$lib/components/page-states/PageErrorState.svelte';
  import PageLoadingState from '$lib/components/page-states/PageLoadingState.svelte';
  import type { Tag } from '$lib/types/tag';
  import { createQuery } from '@tanstack/svelte-query';
  import TagsPage from './TagsPage.svelte';

  const tagsQuery = createQuery<Array<Tag>>({
    queryKey: [queryKeys.tag.list],
    queryFn: () => getTags({ fetch }),
  });
  const tags = $derived($tagsQuery.data);
</script>

{#if $tagsQuery.isPending}
  <PageLoadingState />
{:else if $tagsQuery.isError}
  <PageErrorState error={$tagsQuery.error} />
{:else}
  <TagsPage tags={tags ?? []} />
{/if}
