<script setup lang="ts">
// Renders page-specific actions into the shell's top app bar.
</script>

<template>
  <Teleport defer to="#app-bar-actions">
    <slot />
  </Teleport>
</template>
