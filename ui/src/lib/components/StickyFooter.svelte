<script lang="ts">
  import { onMount } from 'svelte';

  let element: HTMLElement;
  let isFloating = false;

  onMount(() => {
    function onScroll() {
      const offsetBottom = window.innerHeight - element.getBoundingClientRect().bottom;
      console.log(offsetBottom);
      isFloating = offsetBottom <= 0;
    }

    window.addEventListener('scroll', onScroll);
    window.addEventListener('resize', onScroll);

    onScroll();
    return () => {
      window.removeEventListener('scroll', onScroll);
      window.removeEventListener('resize', onScroll);
    };
  });
</script>

<div
  bind:this={element}
  class="w-full sticky bottom-0 transition z-10"
  class:bg-base-200={isFloating}
  class:shadow={isFloating}
>
  <slot />
</div>
