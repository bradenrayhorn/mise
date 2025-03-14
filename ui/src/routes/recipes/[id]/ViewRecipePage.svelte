<script lang="ts">
  import IconLeft from '~icons/mdi/chevron-left';
  import IconPencil from '~icons/mdi/pencil-outline';
  import SingleTag from '$lib/components/SingleTag.svelte';
  import type { DetailedRecipe } from '$lib/types/recipe';
  import { uid } from 'uid';
  import { sticky } from '$lib/actions/sticky';

  type Props = {
    recipe: DetailedRecipe;
    backURL: string;
    id: string;
  };

  const { recipe, backURL, id }: Props = $props();

  const ingredientsSectionID = uid();
  const instructionsSectionID = uid();
</script>

<div class="w-full h-full flex flex-col pbsafe-8">
  <div
    use:sticky
    class="flex flex-col sticky bg-base-500 top-0 z-10 px-4 md:px-8 xl:px-12 pb-4 ptsafe-4 transition-all stuck:shadow-md stuck:bg-base-600 stuck:ptsafe-0 stuck:-translate-y-6 md:stuck:translate-y-0 md:stuck:ptsafe-4"
  >
    <div
      class="flex justify-between items-center mb-4 transition-transform stuck:scale-y-0 md:stuck:scale-y-100"
    >
      <a href={backURL} class="btn-link text-sm text-fg-muted flex items-center gap-1">
        <IconLeft />
        Back
      </a>
      <a
        href={`/recipes/${id}/edit`}
        class="btn-link text-sm text-fg-muted flex items-center gap-2"
      >
        Edit
        <IconPencil />
      </a>
    </div>

    <h1 class="text-xl md:text-3xl font-serif text-fg-highlight font-bold">
      {recipe.title}
    </h1>
  </div>

  <div class="grid grid-cols-1 lg:grid-cols-2 gap-10 px-4 md:px-8 xl:px-12 mt-4">
    {#if recipe.image_id || (recipe.tags.length ?? 0) > 0 || recipe.rich_notes}
      <div class="flex-1 flex flex-col lg:col-span-2">
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
      <h2
        class="text-xl font-bold mb-4 md:mb-6 border-b-2 pb-1 border-b-alpha-200"
        id={ingredientsSectionID}
      >
        Ingredients
      </h2>

      <div class="flex flex-col gap-10">
        {#each recipe.ingredient_blocks as b, i (i)}
          {@const id = uid()}
          <div class="flex flex-col">
            {#if b.title}
              <h3 class="font-bold mb-4 text-lg" {id}>
                {b.title}
              </h3>
            {/if}

            <ul
              class="list-disc ml-4 flex flex-col gap-2"
              aria-labelledby={b.title ? id : ingredientsSectionID}
            >
              {#each b.ingredients as ingredient, j (j)}
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
      <h2
        class="text-xl font-bold mb-4 md:mb-6 border-b-2 pb-1 border-b-alpha-200"
        id={instructionsSectionID}
      >
        Directions
      </h2>

      <div class="flex flex-col gap-10">
        {#each recipe.instruction_blocks as b, i (i)}
          {@const id = uid()}
          <div class="flex flex-col">
            {#if b.title}
              <h3 class="font-bold mb-4 text-lg" {id}>
                {b.title}
              </h3>
            {/if}

            <ol
              class="list-none flex flex-col gap-6"
              aria-labelledby={b.title ? id : instructionsSectionID}
            >
              {#each b.rich_instructions as instruction, j (j)}
                <li>
                  <div class="float-left font-bold text-xl pr-2 leading-6" aria-hidden="true">
                    {j + 1}.
                  </div>
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
