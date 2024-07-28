<script lang="ts">
  import IconChef from '~icons/mdi/chef-hat';
  import IconSad from '~icons/mdi/emoticon-sad-outline';
  import IconRight from '~icons/mdi/chevron-right';
  import IconAdd from '~icons/mdi/plus';
  import StreamedError from '$lib/components/StreamedError.svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { setQueryParameters } from '$lib/replace-query-parameter';
  import TagFilter from './TagFilter.svelte';
  import RecipeImage from '$lib/components/RecipeImage.svelte';
  import DesktopTagFilter from './DesktopTagFilter.svelte';
  import { createQuery, useQueryClient } from '@tanstack/svelte-query';
  import type { RecipePage } from '$lib/types/recipe';
  import { getRecipe, getRecipes } from '$lib/api/load/recipe';
  import { queryKeys } from '$lib/api/query-keys';
  import PrefetchLink from '$lib/components/PrefetchLink.svelte';

  const client = useQueryClient();

  $: localStorage.setItem('last-recipes-query', $page.url.searchParams.toString());

  $: hasCursor = $page.url.searchParams.has('cursor');
  $: searchParams = $page.url.searchParams;

  $: cursor = $page.url.searchParams.get('cursor') ?? '';
  $: search = $page.url.searchParams.get('search') ?? '';
  $: tags = $page.url.searchParams.get('tags') ?? '';

  $: recipesQuery = createQuery<RecipePage>({
    queryKey: [queryKeys.recipe.list, { cursor, search, tags }],
    queryFn: () => getRecipes({ fetch, cursor, search, tags }),
  });

  let searchValue = $page.url.searchParams.get('search') ?? '';
  $: tagValues = ($page.url.searchParams.get('tags') ?? '').split(',').filter((t) => t);
</script>

<div class="w-full min-h-dvh mx-auto flex flex-col justify-between">
  <div class="p-4 mb-4 flex justify-between items-baseline">
    <h1 class="font-bold text-3xl font-serif">Recipes</h1>

    <a href="/recipes/new" class="text-sm btn-link text-fg-muted flex gap-2 items-center">
      Add<IconAdd aria-hidden="true" />
    </a>
  </div>

  <div class="flex grow">
    <div class="px-6 py-4 flex-col w-80 hidden md:flex">
      <div class="font-bold mb-4">Filter</div>
      <DesktopTagFilter
        defaultTagSet={tagValues}
        on:applied={(event) => {
          goto(
            `/recipes?${setQueryParameters(searchParams, { cursor: '', tags: event.detail.tag_ids.join(',') })}`,
            {
              invalidateAll: false,
            },
          );
        }}
      />
    </div>

    <div class="flex flex-col justify-between h-full grow">
      <div>
        <div class="flex gap-2 px-4 md:py-4">
          <form
            class="grow"
            method="POST"
            on:submit={(e) => {
              e.preventDefault();
              goto(
                `/recipes?${setQueryParameters(searchParams, { cursor: '', search: searchValue })}`,
              );
            }}
          >
            <input
              type="text"
              class="input"
              aria-label="Search"
              placeholder="Search"
              bind:value={searchValue}
            />

            <button type="submit" class="hidden">Search</button>
          </form>

          <div class="items-center flex md:hidden">
            <TagFilter
              defaultTagSet={tagValues}
              on:applied={(event) => {
                goto(
                  `/recipes?${setQueryParameters(searchParams, { cursor: '', tags: event.detail.tag_ids.join(',') })}`,
                  {
                    invalidateAll: false,
                  },
                );
              }}
            />
          </div>
        </div>

        <div class="flex flex-col">
          {#if $recipesQuery.status === 'pending'}
            <div>Loading...</div>
          {:else if $recipesQuery.status === 'error'}
            <StreamedError error={$recipesQuery.error}>Could not load recipes.</StreamedError>
          {:else}
            <div class="flex flex-col pl-4 md:px-4">
              {#each $recipesQuery.data.data as recipe (recipe.id)}
                <PrefetchLink
                  class="flex items-center border-b-divider-default border-b py-3 pr-4 md:px-4 hover:bg-base-600 hover:shadow transition data-[loading]:bg-base-primaryHover data-[loading]:animate-pulse"
                  href={`/recipes/${recipe.id}`}
                  prefetch={async () => {
                    await client.prefetchQuery({
                      queryKey: queryKeys.recipe.get(recipe.id),
                      queryFn: () => getRecipe({ fetch, id: recipe.id }),
                    });
                  }}
                >
                  {#if recipe.image_id}
                    <img
                      src={`/api/v1/images/${recipe.image_id}`}
                      alt={recipe.title}
                      class="w-16 h-16 shrink-0 object-cover shadow-inner rounded"
                    />
                  {:else}
                    <div class="w-16 h-16 shrink-0 rounded overflow-hidden text-4xl">
                      <RecipeImage title={recipe.title} />
                    </div>
                  {/if}
                  <span class="ml-4 font-semibold line-clamp-2">{recipe.title}</span>
                </PrefetchLink>
              {:else}
                <div class="flex flex-col items-center pt-12">
                  <div class="flex flex-col items-center justify-center text-fg-muted mb-6">
                    <IconChef class="text-6xl -mb-4 -ml-1" />
                    <IconSad class="text-5xl" />
                  </div>
                  <span>Sorry, Chef.</span>
                  <span>No recipes found.</span>
                </div>
              {/each}
            </div>

            <div class="w-full flex justify-between py-3 px-4">
              {#if hasCursor}
                <a
                  href={`/recipes?${setQueryParameters(searchParams, { cursor: '' })}`}
                  class="text-sm btn-link">Back to first</a
                >
              {:else}
                <div />
              {/if}

              {#if $recipesQuery.data.next}
                <a
                  class="flex items-center text-sm btn-link"
                  href={`/recipes?${setQueryParameters(searchParams, { cursor: $recipesQuery.data.next })}`}
                  >Next<IconRight /></a
                >
              {:else}
                <div />
              {/if}
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>
  <div class="w-full px-4 py-4 mt-3 flex justify-between text-xs text-fg-muted">
    <div>mise</div>
    <a href="/settings" class="btn-link">Settings</a>
  </div>
</div>
