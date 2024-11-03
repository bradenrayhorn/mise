import type { Action } from 'svelte/action';

export const sticky: Action<HTMLElement> = (node) => {
  let isStuck = window.scrollY > 0;
  node.toggleAttribute('data-stuck', isStuck);

  function onScroll() {
    if (window.scrollY > 0 !== isStuck) {
      isStuck = !isStuck;
      requestAnimationFrame(() => {
        node.toggleAttribute('data-stuck', isStuck);
        for (const child of node.children) {
          child.toggleAttribute('data-stuck', isStuck);
        }
      });
    }
  }

  window.addEventListener('scroll', onScroll, { passive: true });

  return {
    destroy() {
      window.removeEventListener('scroll', onScroll);
    },
  };
};
