<script lang="ts">
  import IconTrash from '~icons/mdi/trash-outline';
  import { type SuperForm, type Infer, arrayProxy } from 'sveltekit-superforms';
  import { type RecipeFormSchema } from './schema';
  import type { InstructionBlock } from '$lib/types/recipe';

  interface Props {
    superform: SuperForm<Infer<RecipeFormSchema>>;
  }

  const { superform }: Props = $props();

  const { values: blocks } = arrayProxy(superform, 'instruction_blocks');

  function makeInstructionLabel(blocks: InstructionBlock[], i: number, j: number) {
    if (blocks.length === 1) {
      return `Direction ${j + 1}`;
    } else {
      const block = blocks[i]?.title ?? `Section ${i + 1}`;
      return `${block} direction ${j + 1}`;
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
            aria-label={`Delete ${block.title ? `${block.title} directions` : `directions section ${i + 1}`}`}
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
              aria-label={`Directions section ${i + 1} title`}
            />
          </label>
        </div>
      {/if}

      <div class="flex flex-col gap-2">
        {#each $blocks[i].instructions as _, j (j)}
          <textarea
            class="input resize-none h-36"
            bind:value={$blocks[i].instructions[j]}
            aria-label={makeInstructionLabel($blocks, i, j)}
            onpaste={(e) => {
              const pastedData = e.clipboardData?.getData('Text') ?? '';

              const lines = pastedData
                .split('\n')
                .map((line) => line.replace(/\d+\./g, '').trim())
                .filter((line) => line);
              if (pastedData.trim().length > 0 && lines.length > 1) {
                e.preventDefault();
                const instructions = [...$blocks[i].instructions];
                instructions.splice(j, 0, ...lines);
                $blocks[i].instructions = instructions;
              }
            }}
            oninput={(e) => {
              const currentInstructions = $blocks[i].instructions;
              if (
                e.currentTarget.value?.trim()?.length > 0 &&
                currentInstructions.length === j + 1
              ) {
                $blocks[i].instructions = [...currentInstructions, ''];
              }
            }}
            onkeydown={(e) => {
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
          ></textarea>
        {/each}
      </div>
    </div>
  {/each}
</div>

<button
  class="text-sm text-fg-muted text-right mt-4 w-full"
  onclick={(e) => {
    e.preventDefault();
    $blocks = [...$blocks, { title: '', instructions: [''] }];
  }}>Add additional section</button
>
