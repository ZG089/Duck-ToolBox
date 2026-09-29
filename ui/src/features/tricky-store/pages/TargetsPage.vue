<script setup lang="ts">
import { useMutation } from "@tanstack/vue-query"
import { PackageSearch, Search } from "@lucide/vue"
import { onKeyStroke } from "@vueuse/core"
import { computed, ref, useTemplateRef, watch } from "vue"

import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty"
import { InputGroup, InputGroupAddon, InputGroupInput } from "@/components/ui/input-group"
import AppBarActions from "@/core/components/AppBarActions.vue"
import QueryState from "@/core/components/QueryState.vue"
import { useRouteModal } from "@/core/modal"
import { notifySuccess } from "@/core/notify"

import { trickyStoreApi } from "../api"
import AppRow from "../components/AppRow.vue"
import BackendCard from "../components/BackendCard.vue"
import ModeSheet from "../components/ModeSheet.vue"
import SaveFab from "../components/SaveFab.vue"
import SystemAppsDialog from "../components/SystemAppsDialog.vue"
import TargetsMenu from "../components/TargetsMenu.vue"
import { useTrickyI18n } from "../i18n"
import { useLabels, useStatusQuery } from "../queries"
import { listedApps, matches } from "../selection"
import { useTrickyStore } from "../store"

const { t } = useTrickyI18n()
const store = useTrickyStore()
const status = useStatusQuery()
const modeSheet = useRouteModal("mode")
const systemApps = useRouteModal("system-apps")

watch(status.data, (next) => next && store.load(next), { immediate: true })

const allPackages = computed(() => {
  const names = new Set(store.status?.packages.map((entry) => entry.package_name))
  for (const name of store.draft?.targets.keys() ?? []) names.add(name)
  return [...names].sort()
})
const labels = useLabels(allPackages)

const query = ref("")
const search = useTemplateRef<{ $el: HTMLElement }>("search")
const listed = computed(() =>
  store.status && store.draft ? listedApps(store.status, store.draft, labels.value) : [],
)
const visible = computed(() => listed.value.filter((item) => matches(item, query.value)))
const tunable = computed(
  () => !!store.schema && (store.schema.supports_app_mode || store.schema.supports_per_app_policy),
)

function counted(changed: number, key: "list.selectedCount" | "list.deselectedCount") {
  notifySuccess(changed ? t(key, { count: changed }) : t("list.nothingChanged"))
}

/** With a search active, bulk actions apply to the matching apps only. */
const scope = () => visible.value.map((item) => item.packageName)
const selectAll = () => store.selectMany(scope())
function deselectAll() {
  store.deselectMany(query.value.trim() ? scope() : [...(store.draft?.targets.keys() ?? [])])
}

const save = useMutation({
  mutationFn: async () => {
    const saved = await store.save()
    const next = await status.refetch()
    if (next.data) store.load(next.data, true)
    return saved
  },
  meta: { errorTitle: () => t("ta.prompt_save_error") },
  onSuccess: (saved) =>
    notifySuccess(
      t("ta.prompt_saved_target"),
      saved.restart_required ? t("policy.restart") : undefined,
    ),
})

async function refresh() {
  const next = await status.refetch()
  if (next.data) store.load(next.data, true)
  window.scrollTo({ top: 0 })
}

const denylist = useMutation({
  mutationFn: trickyStoreApi.denylist,
  onSuccess: (data) => counted(store.selectMany(data.packages), "list.selectedCount"),
})
const unnecessary = useMutation({
  mutationFn: async () => {
    const [remote, xposed] = await Promise.all([
      trickyStoreApi.unnecessary(),
      trickyStoreApi.xposed(),
    ])
    return [...remote.packages, ...xposed.packages]
  },
  onSuccess: (packages) => counted(store.deselectMany(packages), "list.deselectedCount"),
})
const autoAdd = useMutation({
  mutationFn: (enabled: boolean) => store.saveOnly({ autoAdd: enabled }),
  meta: { errorTitle: () => t("ta.prompt_save_error") },
})

function discard() {
  if (store.status) store.load(store.status, true)
  notifySuccess(t("list.discarded"))
}

// Tricky Addon's keyboard shortcuts.
const typing = (event: KeyboardEvent) =>
  event.target instanceof HTMLElement && !!event.target.closest("input, textarea")
function shortcut(key: string, action: () => void) {
  onKeyStroke(key, (event) => {
    if (!event.ctrlKey || !store.status?.active || (key !== "f" && key !== "s" && typing(event)))
      return
    event.preventDefault()
    action()
  })
}
shortcut("a", selectAll)
shortcut("d", deselectAll)
shortcut("f", () => search.value?.$el.querySelector("input")?.focus())
shortcut("s", () => save.mutate())
onKeyStroke("Escape", (event) => {
  if (typing(event) && query.value) query.value = ""
})
</script>

<template>
  <div class="flex flex-col gap-3 p-4 pb-32">
    <AppBarActions>
      <TargetsMenu
        v-if="store.status?.active"
        :denylist="store.status.root_manager?.kind === 'magisk'"
        :auto-add="store.draft?.autoAdd ?? false"
        :has-policy="!!store.schema?.default_policy.length"
        @select-all="selectAll"
        @deselect-all="deselectAll"
        @refresh="refresh"
        @denylist="denylist.mutate()"
        @unnecessary="unnecessary.mutate()"
        @system-apps="systemApps.show()"
        @auto-add="autoAdd.mutate($event)"
      />
    </AppBarActions>

    <QueryState
      :loading="status.isPending.value"
      :error="status.error.value"
      @retry="status.refetch()"
    >
      <template v-if="store.status && store.draft">
        <Empty v-if="!store.status.active">
          <EmptyHeader>
            <EmptyMedia variant="icon"><PackageSearch /></EmptyMedia>
            <EmptyTitle>{{ t("backend.none") }}</EmptyTitle>
            <EmptyDescription>{{ t("backend.noneHint") }}</EmptyDescription>
          </EmptyHeader>
        </Empty>

        <template v-else>
          <BackendCard :status="store.status" />

          <!-- Material 3 search bar: 56dp, fully round, filled; it stays under the app bar. -->
          <div class="bg-background sticky top-[calc(4rem+var(--inset-top))] z-20 -mx-4 px-4 py-2">
            <InputGroup ref="search" class="bg-card h-14 rounded-full border-0">
              <InputGroupAddon class="ps-5"><Search class="size-6" /></InputGroupAddon>
              <InputGroupInput
                v-model="query"
                type="search"
                :placeholder="t('ta.search_bar_search_placeholder')"
                :title="t('list.shortcuts')"
              />
              <InputGroupAddon
                align="inline-end"
                class="text-muted-foreground pe-5 text-xs tabular-nums"
              >
                {{ t("list.selected", { count: store.draft.targets.size }) }}
              </InputGroupAddon>
            </InputGroup>
          </div>

          <div class="flex flex-col gap-0.5">
            <AppRow
              v-for="item in visible"
              :key="item.packageName"
              :package-name="item.packageName"
              :label="item.label"
              :system="item.system"
              :mode="store.draft.targets.get(item.packageName) ?? null"
              :custom-policy="store.draft.perApp.has(item.packageName)"
              :tunable="tunable"
              @toggle="store.toggle(item.packageName)"
              @tune="modeSheet.show({ pkg: item.packageName })"
            />
          </div>
          <p v-if="!visible.length" class="text-muted-foreground py-8 text-center text-sm">
            {{ t("list.empty") }}
          </p>
        </template>
      </template>
    </QueryState>

    <template v-if="store.status?.active">
      <SaveFab
        :dirty="store.dirty"
        :saving="save.isPending.value"
        @save="save.mutate()"
        @discard="discard"
      />
      <ModeSheet :labels="labels" />
      <SystemAppsDialog :labels="labels" />
    </template>
  </div>
</template>
