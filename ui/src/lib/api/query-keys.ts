export const queryKeys = {
  recipe: {
    list: 'list-recipes',
    get: (id: string) => ['get-recipe', id],
  },

  tag: {
    list: 'list-tags',
  },

  user: {
    self: 'get-self',
  },
};
