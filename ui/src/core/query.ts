import { MutationCache, QueryClient } from "@tanstack/vue-query"

// A type alias, not an interface: TanStack only accepts meta types assignable to a record.
type DuckMutationMeta = {
  /** Replaces the generic error title in the failure toast. */
  errorTitle?: () => string
}

declare module "@tanstack/vue-query" {
  interface Register {
    mutationMeta: DuckMutationMeta
  }
}

export interface QueryHooks {
  /** Runs after any successful mutation, whichever feature issued it. */
  onMutationSuccess?: () => void
  onMutationError?: (error: unknown, title?: string) => void
}

export function createQueryClient(hooks: QueryHooks = {}): QueryClient {
  return new QueryClient({
    mutationCache: new MutationCache({
      onSuccess: () => hooks.onMutationSuccess?.(),
      onError: (error, _variables, _context, mutation) => {
        if (!mutation.options.onError) {
          hooks.onMutationError?.(error, mutation.options.meta?.errorTitle?.())
        }
      },
    }),
    defaultOptions: {
      queries: {
        // duckd is local: never pause for connectivity, and its errors are deterministic.
        networkMode: "always",
        retry: false,
        refetchOnWindowFocus: false,
        staleTime: 15_000,
      },
      mutations: {
        networkMode: "always",
        retry: false,
      },
    },
  })
}
