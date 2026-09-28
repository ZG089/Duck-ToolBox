<script setup lang="ts">
import { ChevronRight } from "@lucide/vue"
import { computed } from "vue"
import { useI18n } from "vue-i18n"

import { Badge } from "@/components/ui/badge"
import {
  Item,
  ItemActions,
  ItemContent,
  ItemDescription,
  ItemMedia,
  ItemTitle,
} from "@/components/ui/item"
import type { Availability, FeatureDefinition } from "@/core/features"

const props = defineProps<{ feature: FeatureDefinition; availability: Availability }>()
const emit = defineEmits<{ open: [feature: FeatureDefinition] }>()
const { t } = useI18n()

const blocked = computed(
  () => props.availability === "missing" || props.availability === "outdated",
)
const badge = computed(() =>
  props.availability === "missing"
    ? t("core.home.missing")
    : props.availability === "outdated"
      ? t("core.home.outdated")
      : null,
)
</script>

<template>
  <Item
    as="button"
    variant="outline"
    class="bg-card w-full text-start disabled:opacity-60"
    :disabled="blocked"
    @click="emit('open', feature)"
  >
    <ItemMedia variant="icon" class="bg-secondary text-secondary-foreground size-10 rounded-xl">
      <component :is="feature.icon" class="size-5" />
    </ItemMedia>
    <ItemContent class="min-w-0">
      <ItemTitle>{{ t(`${feature.namespace}.meta.title`) }}</ItemTitle>
      <ItemDescription class="line-clamp-2">
        {{ t(`${feature.namespace}.meta.description`) }}
      </ItemDescription>
    </ItemContent>
    <ItemActions>
      <Badge v-if="badge" variant="secondary">{{ badge }}</Badge>
      <ChevronRight v-else class="text-muted-foreground rtl-flip size-4" />
    </ItemActions>
  </Item>
</template>
