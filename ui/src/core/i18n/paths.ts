/** Dot-separated paths to every string leaf of a message object, e.g. `menu.keybox`. */
export type DotPath<T> = T extends string
  ? never
  : {
      [K in keyof T & string]: T[K] extends string ? K : `${K}.${DotPath<T[K]>}`
    }[keyof T & string]
