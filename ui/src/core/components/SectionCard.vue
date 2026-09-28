<script setup lang="ts">
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"

defineProps<{
  title?: string
  description?: string
  contentClass?: string
}>()
</script>

<template>
  <Card class="gap-4 py-4">
    <CardHeader v-if="title || description || $slots.action" class="px-4">
      <CardTitle v-if="title" class="text-base">{{ title }}</CardTitle>
      <CardDescription v-if="description">{{ description }}</CardDescription>
      <CardAction v-if="$slots.action">
        <slot name="action" />
      </CardAction>
    </CardHeader>
    <CardContent :class="['px-4', contentClass]">
      <slot />
    </CardContent>
  </Card>
</template>
