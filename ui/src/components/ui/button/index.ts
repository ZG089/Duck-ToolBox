import type { VariantProps } from "class-variance-authority"
import { cva } from "class-variance-authority"

export { default as Button } from "./Button.vue"

// Material 3 Expressive buttons: round at rest, squarer while pressed, 48dp touch targets.
export const buttonVariants = cva(
  "state-layer touch-target relative inline-flex shrink-0 items-center justify-center gap-2 font-medium whitespace-nowrap select-none outline-none transition-[border-radius] duration-350 ease-spring-fast disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-5 focus-visible:ring-3 focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive",
  {
    variants: {
      variant: {
        default: "bg-primary text-primary-foreground",
        destructive:
          "bg-destructive text-white focus-visible:ring-destructive/20 dark:focus-visible:ring-destructive/40 dark:bg-destructive/60",
        outline: "border border-foreground/20 bg-transparent",
        secondary: "bg-secondary text-secondary-foreground",
        ghost: "",
        link: "text-primary underline-offset-4 hover:underline",
      },
      size: {
        "default": "h-10 rounded-xl px-4 text-sm active:rounded-sm",
        "xs": "h-8 gap-1.5 rounded-lg px-3 text-xs active:rounded-sm [&_svg:not([class*='size-'])]:size-4",
        "sm": "h-8 gap-1.5 rounded-lg px-3 text-sm active:rounded-sm [&_svg:not([class*='size-'])]:size-4.5",
        "lg": "h-14 rounded-2xl px-6 text-base active:rounded-md [&_svg:not([class*='size-'])]:size-6",
        "icon": "size-10 rounded-xl active:rounded-sm [&_svg:not([class*='size-'])]:size-6",
        "icon-xs": "size-8 rounded-lg active:rounded-sm [&_svg:not([class*='size-'])]:size-4",
        "icon-sm": "size-8 rounded-lg active:rounded-sm",
        "icon-lg": "size-14 rounded-2xl active:rounded-md [&_svg:not([class*='size-'])]:size-6",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  },
)
export type ButtonVariants = VariantProps<typeof buttonVariants>
