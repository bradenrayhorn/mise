<script lang="ts">
  import Icon1 from '~icons/mdi/local-dining';
  import Icon2 from '~icons/mdi/baguette';
  import Icon3 from '~icons/mdi/food-apple-outline';
  import Icon4 from '~icons/mdi/carrot';
  import Icon5 from '~icons/mdi/cheese';
  import Icon6 from '~icons/mdi/food-turkey';
  import Icon7 from '~icons/mdi/pot-steam-outline';
  import Icon8 from '~icons/mdi/hamburger';
  import Icon9 from '~icons/mdi/fruit-watermelon';
  import Icon10 from '~icons/mdi/food-drumstick-outline';

  const colorsMap = [
    'bg-food-1',
    'bg-food-2',
    'bg-food-3',
    'bg-food-4',
    'bg-food-5',
    'bg-food-6',
    'bg-food-7',
    'bg-food-8',
    'bg-food-9',
    'bg-food-10',
  ];

  const iconsMap = [Icon1, Icon2, Icon3, Icon4, Icon5, Icon6, Icon7, Icon8, Icon9, Icon10];

  interface Props {
    title: string;
  }

  const { title }: Props = $props();

  function hashTitle(title: string): number {
    let hash = 0;
    for (let i = 0; i < title.length; i++) {
      const char = title.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash |= 0;
    }
    return Math.abs(hash);
  }

  function randomIndexes(hash: number): Array<number> {
    return [hash % 10, (hash >> 3) % 10];
  }

  const indexes = $derived(randomIndexes(hashTitle(title)));

  const SvelteComponent = $derived(iconsMap[indexes[1]]);
</script>

<div
  class={`w-full h-full ${colorsMap[indexes[0]]} flex items-center justify-center text-[rgba(255,255,255,0.72)]`}
>
  <SvelteComponent />
</div>
