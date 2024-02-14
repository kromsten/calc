export const fetchAllPaginatedResources = async function <G, T>(
  requestFn: (...args: any[]) => Promise<G>,
  getResponseItems: (response: G) => T[],
  getNextPageValue: (response: G | undefined) => unknown | undefined,
): Promise<T[]> {
  const results = [];
  const generator = fetchNextPaged(requestFn, getResponseItems, getNextPageValue);

  for await (const page of generator) {
    results.push(...page);
  }

  return results;
};

const fetchNextPaged = async function* <G, T>(
  requestFn: (...args: any[]) => Promise<G>,
  getResponseItems: (response: G) => T[],
  getNextPageValue: (response: G | undefined) => unknown | undefined,
): AsyncGenerator<T[]> {
  async function* makeRequest<G, T>(
    requestFn: (...args: any[]) => Promise<G>,
    getResponseItems: (response: G) => T[],
    getNextPageValue: (response: G | undefined) => unknown | undefined,
    previousPage: G | undefined = undefined,
  ): AsyncGenerator<T[]> {
    const response = await requestFn(getNextPageValue(previousPage));
    yield getResponseItems(response);

    if (getNextPageValue(response) !== undefined) {
      yield* makeRequest(requestFn, getResponseItems, getNextPageValue, response);
    }
  }

  yield* makeRequest(requestFn, getResponseItems, getNextPageValue);
};
