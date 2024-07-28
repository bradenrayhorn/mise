<script lang="ts">
  import { page } from '$app/stores';
  import { createQuery } from '@tanstack/svelte-query';
  import type { DetailedRecipeWithHash } from '$lib/types/recipe';
  import { queryKeys } from '$lib/api/query-keys';
  import { getRecipe } from '$lib/api/load/recipe';
  import ViewRecipePage from './ViewRecipePage.svelte';
  import PageLoadingState from '$lib/components/page-states/PageLoadingState.svelte';
  import PageErrorState from '$lib/components/page-states/PageErrorState.svelte';

  const backURL = `/recipes?${localStorage.getItem('last-recipes-query')}`;
  const id = $page.params['id'];

  $: query = createQuery<DetailedRecipeWithHash>({
    queryKey: queryKeys.recipe.get(id),
    queryFn: () => getRecipe({ fetch, id }),
  });
</script>

{#if $query.isPending}
  <PageLoadingState />
{:else if $query.isError}
  <PageErrorState error={$query.error} />
{:else}
  <ViewRecipePage {backURL} {id} recipe={$query.data.recipe} />
{/if}
