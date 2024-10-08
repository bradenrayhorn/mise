<script lang="ts">
  import SingleTag from '$lib/components/SingleTag.svelte';
  import CloseIconButton from '$lib/components/CloseIconButton.svelte';
  import type { DetailedRecipe } from '$lib/types/recipe';
  import { uid } from 'uid';

  export let recipe: DetailedRecipe;
  export let backURL: string;
  export let id: string;

  const ingredientsSectionID = uid();
  const instructionsSectionID = uid();
</script>

<div class="absolute top-1 left-1 z-10 flex items-center">
  <CloseIconButton href={backURL} />
</div>

<div class="w-full h-full flex flex-col pb-8">
  <div class="flex justify-between items-baseline px-4 md:px-8 lg:px-12 pb-8 pt-12">
    <h1 class="text-3xl font-serif font-bold pr-4">
      {recipe.title}
    </h1>

    <a href={`/recipes/${id}/edit`} class="btn-link text-sm text-fg-muted">Edit</a>
  </div>

  <div class="flex flex-col md:flex-row gap-8 px-4 md:px-8 lg:px-12">
    {#if recipe.image_id || (recipe.tags.length ?? 0) > 0 || recipe.rich_notes}
      <div class="flex-1 flex flex-col">
        {#if recipe.image_id}
          <img
            src={`/api/v1/images/${recipe.image_id}`}
            alt={recipe.title}
            class="w-full max-w-80 h-56 object-cover rounded shadow-inner"
          />
        {/if}

        <ul class="flex my-6 flex-wrap gap-2" aria-label="Tags">
          {#each recipe.tags as tag (tag.id)}
            <SingleTag name={tag.name} canDelete={false} />
          {/each}
        </ul>

        {#if recipe.rich_notes}
          <section class="prose" aria-label="Notes">
            <!-- eslint-disable svelte/no-at-html-tags -->
            {@html recipe.rich_notes}
          </section>
        {/if}
      </div>
    {/if}

    <section class="flex-1" aria-labelledby={ingredientsSectionID}>
      <h2 class="text-xl font-bold mb-4 md:mb-6" id={ingredientsSectionID}>Ingredients</h2>

      <div class="flex flex-col gap-4">
        {#each recipe.ingredient_blocks as b}
          {@const id = uid()}
          <div class="flex flex-col">
            {#if b.title}
              <h3 class="font-bold mb-2" {id}>
                {b.title}
              </h3>
            {/if}

            <ul
              class="list-disc ml-4 flex flex-col gap-1"
              aria-labelledby={b.title ? id : ingredientsSectionID}
            >
              {#each b.ingredients as ingredient}
                <li>
                  {ingredient}
                </li>
              {/each}
            </ul>
          </div>
        {/each}
      </div>
    </section>

    <section class="flex-1" aria-labelledby={instructionsSectionID}>
      <h2 class="text-xl font-bold mb-4 md:mb-6" id={instructionsSectionID}>Instructions</h2>

      <div class="flex flex-col gap-4">
        {#each recipe.instruction_blocks as b}
          {@const id = uid()}
          <div class="flex flex-col">
            {#if b.title}
              <h3 class="font-bold mb-2" {id}>
                {b.title}
              </h3>
            {/if}

            <ol
              class="list-decimal ml-4 flex flex-col gap-1"
              aria-labelledby={b.title ? id : instructionsSectionID}
            >
              {#each b.rich_instructions as instruction}
                <li>
                  <!-- eslint-disable svelte/no-at-html-tags -->
                  <div class="prose">{@html instruction}</div>
                </li>
              {/each}
            </ol>
          </div>
        {/each}
      </div>
    </section>
  </div>
</div>
