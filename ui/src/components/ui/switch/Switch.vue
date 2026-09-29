<script setup lang="ts">
import type { SwitchRootEmits, SwitchRootProps } from "reka-ui"
import type { HTMLAttributes } from "vue"
import { reactiveOmit } from "@vueuse/core"
import {
  SwitchRoot,
  SwitchThumb,
  useForwardPropsEmits,
} from "reka-ui"
import { cn } from "@/lib/utils"

const props = defineProps<SwitchRootProps & { class?: HTMLAttributes["class"] }>()

const emits = defineEmits<SwitchRootEmits>()

const delegatedProps = reactiveOmit(props, "class")

const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <!-- Material 3 switch: 52 × 32 track, 16dp handle that grows to 24dp on and 28dp while pressed. -->
  <SwitchRoot
    v-slot="slotProps"
    data-slot="switch"
    v-bind="forwarded"
    :class="cn(
      'group/switch peer touch-target relative inline-flex h-8 w-13 shrink-0 items-center rounded-full border-2 transition-colors outline-none focus-visible:ring-3 focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50',
      'data-[state=unchecked]:border-input data-[state=unchecked]:bg-muted data-[state=checked]:border-primary data-[state=checked]:bg-primary',
      props.class,
    )"
  >
    <!-- Logical margins place the handle, so it mirrors in right-to-left layouts by itself. -->
    <SwitchThumb
      data-slot="switch-thumb"
      :class="cn(
        'pointer-events-none flex items-center justify-center rounded-full transition-[margin,width,height,background-color] duration-350 ease-spring-fast',
        'bg-input ms-1.5 size-4 data-[state=checked]:bg-primary-foreground data-[state=checked]:ms-5.5 data-[state=checked]:size-6',
        'group-active/switch:ms-0 group-active/switch:size-7 group-active/switch:data-[state=checked]:ms-5 group-active/switch:data-[state=checked]:size-7',
      )"
    >
      <slot name="thumb" v-bind="slotProps" />
    </SwitchThumb>
  </SwitchRoot>
</template>
