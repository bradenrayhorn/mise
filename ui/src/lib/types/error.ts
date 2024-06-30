export type MaybeError =
  | {
      status?: number;
      location?: string;
      message?: string;
    }
  | undefined;
