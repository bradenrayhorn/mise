export function streamedPromise<T>(promise: Promise<T>): Promise<T> {
  return promise.catch((error) => {
    throw error;
  });
}
