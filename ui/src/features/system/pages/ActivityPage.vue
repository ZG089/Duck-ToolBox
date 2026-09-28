<script setup lang="ts">
import ActivityList from "../components/ActivityList.vue"
</script>

<template>
  <div class="p-4">
    <ActivityList :limit="200" />
  </div>
</template>
