import type { VariantProps } from "class-variance-authority"
import { cva } from "class-variance-authority"

export { default as Item } from "./Item.vue"
export { default as ItemActions } from "./ItemActions.vue"
export { default as ItemContent } from "./ItemContent.vue"
export { default as ItemDescription } from "./ItemDescription.vue"
export { default as ItemFooter } from "./ItemFooter.vue"
export { default as ItemGroup } from "./ItemGroup.vue"
export { default as ItemHeader } from "./ItemHeader.vue"
export { default as ItemMedia } from "./ItemMedia.vue"
export { default as ItemSeparator } from "./ItemSeparator.vue"
export { default as ItemTitle } from "./ItemTitle.vue"

export const itemVariants = cva(
  "group/item flex items-center border border-transparent text-sm rounded-md transition-[border-radius,background-color] duration-350 ease-spring-fast flex-wrap outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-3",
  {
    variants: {
      variant: {
        default: "bg-transparent",
        outline: "border-border",
        muted: "bg-muted/50",
        // Material 3 Expressive segmented list: 4dp inner corners, 16dp outer ones.
        segmented: "bg-card rounded-xs first:rounded-t-lg last:rounded-b-lg",
      },
      size: {
        default: "min-h-18 gap-4 px-4 py-3",
        sm: "min-h-14 gap-4 px-4 py-2",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  },
)

export const itemMediaVariants = cva(
  "flex shrink-0 items-center justify-center gap-2 [&_svg]:pointer-events-none",
  {
    variants: {
      variant: {
        default: "bg-transparent",
        icon: "size-10 rounded-full bg-secondary text-secondary-foreground [&_svg:not([class*='size-'])]:size-5",
        image:
          "size-10 rounded-full overflow-hidden [&_img]:size-full [&_img]:object-cover",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  },
)

export type ItemVariants = VariantProps<typeof itemVariants>
export type ItemMediaVariants = VariantProps<typeof itemMediaVariants>
