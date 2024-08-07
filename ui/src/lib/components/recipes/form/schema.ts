import { z } from 'zod';

export const schema = z
  .object({
    title: z.string().trim().min(1, 'Title is required.'),
    notes: z.string().trim(),
    image: z
      .instanceof(File, { message: 'Must be a file.' })
      .refine((f) => f.size < 10_485_760, 'Max upload size 10MB.')
      .or(z.undefined()),
    ingredient_blocks: z
      .object({
        title: z.string().optional(),
        ingredients: z.string().trim().array(),
      })
      .array(),
    instruction_blocks: z
      .object({
        title: z.string().optional(),
        instructions: z.string().trim().array(),
      })
      .array(),
    tags: z
      .object({
        id: z.string(),
        name: z.string(),
      })
      .required()
      .array(),
  })
  .required();

export type RecipeFormSchema = typeof schema;

export type RecipeFormType = {
  title: string;
  notes: string;
  image?: File;
  ingredient_blocks: Array<{ title?: string; ingredients: Array<string> }>;
  instruction_blocks: Array<{ title?: string; instructions: Array<string> }>;
  tags: Array<{ id: string; name: string }>;
};
