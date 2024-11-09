<script lang="ts">
  import { page } from '$app/stores';
  import { queryKeys } from '$lib/api/query-keys';
  import PageErrorState from '$lib/components/page-states/PageErrorState.svelte';
  import PageLoadingState from '$lib/components/page-states/PageLoadingState.svelte';
  import type { DetailedRecipeWithHash } from '$lib/types/recipe';
  import { createQuery } from '@tanstack/svelte-query';
  import EditRecipePage from './EditRecipePage.svelte';
  import { getRecipe } from '$lib/api/load/recipe';

  const id = $page.params['id'];
  const backURL = `/recipes/${id}`;

  let query = $derived(
    createQuery<DetailedRecipeWithHash>({
      queryKey: queryKeys.recipe.get(id),
      queryFn: () => getRecipe({ fetch, id }),
    }),
  );

  function buildInitialData({ recipe }: DetailedRecipeWithHash) {
    const ingredient_blocks = recipe.ingredient_blocks.map((block) => ({
      title: block.title,
      ingredients: [...block.ingredients, ''],
    }));

    if (ingredient_blocks.length === 1 && ingredient_blocks[0].title) {
      ingredient_blocks.push({ title: undefined, ingredients: [''] });
    }

    const instruction_blocks = recipe.instruction_blocks.map((block) => ({
      title: block.title,
      instructions: [...block.instructions, ''],
    }));

    if (instruction_blocks.length === 1 && instruction_blocks[0].title) {
      instruction_blocks.push({ title: undefined, instructions: [''] });
    }

    return {
      title: recipe.title,
      notes: recipe.notes ?? '',
      image: recipe.image_id ? new File([], recipe.image_id, { type: 'mise/image_id' }) : undefined,
      tags: recipe.tags,
      ingredient_blocks: ingredient_blocks.length > 0 ? ingredient_blocks : [{ ingredients: [''] }],
      instruction_blocks:
        instruction_blocks.length > 0 ? instruction_blocks : [{ instructions: [''] }],
    };
  }

  let initialData = $derived($query.data ? buildInitialData($query.data) : undefined);
</script>

{#if $query.isPending}
  <PageLoadingState />
{:else if $query.isError}
  <PageErrorState error={$query.error} />
{:else if initialData !== undefined}
  <EditRecipePage
    {id}
    {backURL}
    {initialData}
    hash={$query.data.hash}
    recipe={$query.data.recipe}
  />
{/if}
