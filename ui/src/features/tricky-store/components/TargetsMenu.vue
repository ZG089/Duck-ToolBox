<script setup lang="ts">
import {
  BadgeCheck,
  BookOpen,
  CheckCheck,
  EllipsisVertical,
  KeyRound,
  ListMinus,
  ListPlus,
  RefreshCw,
  Settings2,
  ShieldBan,
  SlidersHorizontal,
  SquareDashed,
} from "@lucide/vue"
import { useRouter } from "vue-router"

import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"

import { useTrickyI18n } from "../i18n"

defineProps<{ denylist: boolean; autoAdd: boolean; hasPolicy: boolean }>()
const emit = defineEmits<{
  selectAll: []
  deselectAll: []
  refresh: []
  denylist: []
  unnecessary: []
  systemApps: []
  autoAdd: [enabled: boolean]
}>()
const { t } = useTrickyI18n()
const router = useRouter()
const go = (path: string) => router.push(`/tricky-store/${path}`)
</script>

<template>
  <DropdownMenu>
    <DropdownMenuTrigger as-child>
      <Button variant="ghost" size="icon" :aria-label="t('list.more')">
        <EllipsisVertical />
      </Button>
    </DropdownMenuTrigger>
    <DropdownMenuContent align="end" class="w-60">
      <DropdownMenuItem @select="emit('selectAll')"
        ><CheckCheck />{{ t("list.selectAll") }}</DropdownMenuItem
      >
      <DropdownMenuItem @select="emit('deselectAll')"
        ><SquareDashed />{{ t("list.deselectAll") }}</DropdownMenuItem
      >
      <DropdownMenuItem @select="emit('refresh')"
        ><RefreshCw />{{ t("list.refresh") }}</DropdownMenuItem
      >
      <DropdownMenuSeparator />
      <DropdownMenuItem v-if="denylist" @select="emit('denylist')">
        <ShieldBan />{{ t("ta.menu_select_denylist") }}
      </DropdownMenuItem>
      <DropdownMenuItem @select="emit('unnecessary')"
        ><ListMinus />{{ t("ta.menu_deselect_unnecessary") }}</DropdownMenuItem
      >
      <DropdownMenuItem @select="emit('systemApps')"
        ><ListPlus />{{ t("ta.menu_add_system_app") }}</DropdownMenuItem
      >
      <DropdownMenuCheckboxItem
        :model-value="autoAdd"
        @update:model-value="emit('autoAdd', $event)"
      >
        {{ t("list.autoAdd") }}
      </DropdownMenuCheckboxItem>
      <DropdownMenuSeparator />
      <DropdownMenuItem @select="go('keybox')"
        ><KeyRound />{{ t("ta.menu_keybox") }}</DropdownMenuItem
      >
      <DropdownMenuItem @select="go('props')"
        ><Settings2 />{{ t("ta.menu_prop_setting") }}</DropdownMenuItem
      >
      <DropdownMenuItem v-if="hasPolicy" @select="go('policy')">
        <SlidersHorizontal />{{ t("ta.menu_set_default_policy") }}
      </DropdownMenuItem>
      <DropdownMenuSeparator />
      <DropdownMenuItem @select="go('help')"><BookOpen />{{ t("ta.menu_help") }}</DropdownMenuItem>
      <DropdownMenuItem @select="router.push('/settings')"
        ><BadgeCheck />{{ t("entry.title") }}</DropdownMenuItem
      >
    </DropdownMenuContent>
  </DropdownMenu>
</template>
