import type { TagOnRecipe } from './tag';

export type ListedRecipe = {
  id: string;
  title: string;
  image_id?: string;
};

export type RecipePage = {
  data: Array<ListedRecipe>;
  next?: string;
};

export type DetailedRecipe = {
  id: string;
  title: string;
  image_id?: string;
  notes?: string;
  rich_notes?: string;
  tags: Array<TagOnRecipe>;
  ingredient_blocks: Array<IngredientBlock>;
  instruction_blocks: Array<RichInstructionBlock>;
};

export type DetailedRecipeWithHash = {
  hash: string;
  recipe: DetailedRecipe;
};

export type IngredientBlock = {
  title?: string;
  ingredients: Array<string>;
};

export type InstructionBlock = {
  title?: string;
  instructions: Array<string>;
};

type RichInstructionBlock = InstructionBlock & { rich_instructions: Array<string> };
