<script lang="ts">
  import IconTrash from '~icons/mdi/trash-outline';
  import { type SuperForm, type Infer, arrayProxy } from 'sveltekit-superforms';
  import { type RecipeFormSchema } from './schema';

  export let superform: SuperForm<Infer<RecipeFormSchema>>;

  const { values: blocks } = arrayProxy(superform, 'instruction_blocks');
</script>

<div class="flex flex-col gap-10">
  {#each $blocks as _, i}
    <div>
      {#if $blocks.length > 1}
        <div class="font-serif font-bold mb-4 flex justify-between">
          {$blocks[i].title ? $blocks[i].title : `Component ${i + 1}`}

          <button
            class="text-lg"
            on:click|preventDefault={() => {
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
              aria-label="Title"
            />
          </label>
        </div>
      {/if}

      <div class="flex flex-col gap-2">
        {#each $blocks[i].instructions as _, j}
          <textarea
            class="input resize-none h-36"
            bind:value={$blocks[i].instructions[j]}
            on:input={(e) => {
              const currentInstructions = $blocks[i].instructions;
              if (
                e.currentTarget.value?.trim()?.length > 0 &&
                currentInstructions.length === j + 1
              ) {
                $blocks[i].instructions = [...currentInstructions, ''];
              }
            }}
            on:keydown={(e) => {
              const currentInstructions = $blocks[i].instructions;
              if (
                e.key === 'Backspace' &&
                currentInstructions.length > 1 &&
                currentInstructions.length - 1 !== j &&
                $blocks[i].instructions[j] === ''
              ) {
                const next = [...currentInstructions];
                next.splice(j, 1);
                $blocks[i].instructions = next;
              }
            }}
          />
        {/each}
      </div>
    </div>
  {/each}
</div>

<button
  class="text-sm text-text-300 text-right mt-4 w-full"
  on:click|preventDefault={() => {
    $blocks = [...$blocks, { title: '', instructions: [''] }];
  }}>Add additional component</button
>
