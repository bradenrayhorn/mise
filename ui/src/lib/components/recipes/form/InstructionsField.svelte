<script lang="ts">
  import { type SuperForm, type Infer, arrayProxy } from 'sveltekit-superforms';
  import { type RecipeFormSchema } from './schema';

  export let superform: SuperForm<Infer<RecipeFormSchema>>;

  const { values: blocks } = arrayProxy(superform, 'instruction_blocks');
</script>

{#each $blocks as _, i}
  {#if $blocks.length > 1}
    <div>
      <label>
        <span>Title {i}</span>
        <input class="border-black border" bind:value={$blocks[i].title} />
      </label>

      <button
        on:click|preventDefault={() => {
          const next = [...$blocks];
          next.splice(i, 1);
          $blocks = next;
        }}>X</button
      >
    </div>
  {/if}

  {#each $blocks[i].instructions as _, j}
    <div>
      <textarea
        class="border-black border"
        bind:value={$blocks[i].instructions[j]}
        on:input={(e) => {
          const currentInstructions = $blocks[i].instructions;
          if (e.currentTarget.value?.trim()?.length > 0 && currentInstructions.length === j + 1) {
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
    </div>
  {/each}
{/each}

<button
  on:click|preventDefault={() => {
    $blocks = [...$blocks, { title: '', instructions: [''] }];
  }}>Add additional component</button
>
