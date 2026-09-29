<script setup lang="ts">
import type { TabsTriggerProps } from "reka-ui"
import type { HTMLAttributes } from "vue"
import { reactiveOmit } from "@vueuse/core"
import { TabsTrigger, useForwardProps } from "reka-ui"
import { cn } from "@/lib/utils"

const props = defineProps<TabsTriggerProps & { class?: HTMLAttributes["class"] }>()

const delegatedProps = reactiveOmit(props, "class")

const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <TabsTrigger
    data-slot="tabs-trigger"
    :class="cn(
      `state-layer data-[state=active]:text-foreground focus-visible:ring-ring/50 relative inline-flex flex-1 items-center justify-center gap-1.5 rounded-t-md px-3 text-sm font-medium whitespace-nowrap transition-colors outline-none select-none focus-visible:ring-3 disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-5`,
      // Material 3 primary tab indicator: 3dp, rounded on top.
      `after:bg-primary after:absolute after:inset-x-3 after:bottom-0 after:h-0.75 after:scale-x-0 after:rounded-t-full after:transition-transform after:duration-350 after:ease-spring-fast data-[state=active]:after:scale-x-100`,
      props.class,
    )"
    v-bind="forwardedProps"
  >
    <slot />
  </TabsTrigger>
</template>
