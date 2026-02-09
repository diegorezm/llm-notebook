export type Result<S, E extends { reason: string }> = [E, null] | [null, S];

export function ok<S>(data: S): Result<S, never> {
  return [null, data];
}

export function err<E extends { reason: string }>(error: E): Result<never, E> {
  return [error, null];
}
