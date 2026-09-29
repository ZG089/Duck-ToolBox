import type { VariantProps } from "class-variance-authority"
import { cva } from "class-variance-authority"

export { default as Toggle } from "./Toggle.vue"

export const toggleVariants = cva(
  "state-layer touch-target relative inline-flex items-center justify-center gap-2 text-sm font-medium whitespace-nowrap select-none outline-none transition-[border-radius,background-color,color] duration-350 ease-spring-fast disabled:pointer-events-none disabled:opacity-50 data-[state=on]:bg-primary data-[state=on]:text-primary-foreground [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-5 [&_svg]:shrink-0 focus-visible:ring-3 focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive",
  {
    variants: {
      variant: {
        default: "bg-transparent",
        outline: "border border-foreground/20 bg-transparent data-[state=on]:border-primary",
        tonal: "bg-secondary text-secondary-foreground",
      },
      size: {
        default: "h-10 min-w-10 rounded-xl px-4",
        sm: "h-8 min-w-8 rounded-lg px-3",
        lg: "h-14 min-w-14 rounded-2xl px-6 text-base",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  },
)

export type ToggleVariants = VariantProps<typeof toggleVariants>
