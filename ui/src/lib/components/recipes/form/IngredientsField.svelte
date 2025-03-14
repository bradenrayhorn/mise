<script lang="ts">
  import IconTrash from '~icons/mdi/trash-outline';
  import { type SuperForm, type Infer, arrayProxy } from 'sveltekit-superforms';
  import { type RecipeFormSchema } from './schema';
  import type { IngredientBlock } from '$lib/types/recipe';

  interface Props {
    superform: SuperForm<Infer<RecipeFormSchema>>;
  }

  const { superform }: Props = $props();

  const { values: blocks } = arrayProxy(superform, 'ingredient_blocks');

  function makeIngredientLabel(blocks: IngredientBlock[], i: number, j: number) {
    if (blocks.length === 1) {
      return `Ingredient ${j + 1}`;
    } else {
      const block = blocks[i]?.title ?? `Section ${i + 1}`;
      return `${block} ingredient ${j + 1}`;
    }
  }
</script>

<div class="flex flex-col gap-10">
  {#each $blocks as block, i (i)}
    <div>
      {#if $blocks.length > 1}
        <div class="font-bold mb-4 flex justify-between">
          {block.title ? block.title : `Section ${i + 1}`}

          <button
            class="text-lg"
            aria-label={`Delete ${block.title ? `${block.title} ingredients` : `ingredient section ${i + 1}`}`}
            onclick={(e) => {
              e.preventDefault();
              const next = [...$blocks];
              next.splice(i, 1);
              $blocks = next;
            }}><IconTrash /></button
          >
        </div>

        <div class="mb-8">
          <label>
            <input
              class="input"
              bind:value={$blocks[i].title}
              placeholder="Title"
              aria-label={`Ingredient section ${i + 1} title`}
            />
          </label>
        </div>
      {/if}

      <div class="flex flex-col gap-2">
        {#each $blocks[i].ingredients as _, j (j)}
          <input
            class="input"
            bind:value={$blocks[i].ingredients[j]}
            aria-label={makeIngredientLabel($blocks, i, j)}
            onpaste={(e) => {
              const pastedData = e.clipboardData?.getData('Text') ?? '';

              const lines = pastedData
                .split('\n')
                .map((line) =>
                  line
                    .replace(/\(\$\d+\.\d{2}\)/g, '')
                    .replace(/\$\d+\.\d{2}/g, '')
                    .trim(),
                )
                .filter((line) => line);
              if (pastedData.trim().length > 0 && lines.length > 1) {
                e.preventDefault();
                const ingredients = [...$blocks[i].ingredients];
                ingredients.splice(j, 0, ...lines);
                $blocks[i].ingredients = ingredients;
              }
            }}
            oninput={(e) => {
              const currentIngredients = $blocks[i].ingredients;
              if (
                e.currentTarget.value?.trim()?.length > 0 &&
                currentIngredients.length === j + 1
              ) {
                $blocks[i].ingredients = [...currentIngredients, ''];
              }
            }}
            onkeydown={(e) => {
              const currentIngredients = $blocks[i].ingredients;
              if (
                e.key === 'Backspace' &&
                currentIngredients.length > 1 &&
                currentIngredients.length - 1 !== j &&
                $blocks[i].ingredients[j] === ''
              ) {
                const next = [...currentIngredients];
                next.splice(j, 1);
                $blocks[i].ingredients = next;
              }
            }}
          />
        {/each}
      </div>
    </div>
  {/each}
</div>

<button
  class="text-sm text-fg-muted text-right mt-4 w-full"
  onclick={(e) => {
    e.preventDefault();
    $blocks = [...$blocks, { title: '', ingredients: [''] }];
  }}>Add additional section</button
>
