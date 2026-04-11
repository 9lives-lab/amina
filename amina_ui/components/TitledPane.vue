<template>
  <div class="titled-pane column">
    <div class="header col-auto row text-body2">
      <div class="col">{{ title }}</div>
      <slot name="header" class="col"></slot>
    </div>
    <div class="content column col">
      <slot></slot>
    </div>
  </div>
</template>

<script setup>
defineProps({
  title: String
})
</script>

<style lang="sass" scoped>
.titled-pane
  display: grid
  grid-template-rows: $amina-control-height auto
  .header
    display: flex
    justify-content: left
    align-items: center
    padding-left: $amina-default-padding
    background-color: $amina-primary-color
    border-color: $amina-separator-color
    border-width: 1px 0 1px 0
    border-style: solid

</style>
