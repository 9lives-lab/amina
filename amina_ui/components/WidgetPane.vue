<template>
  <div class="widget-pane column">
    <div v-if="title !== null" class="header col-auto row text-weight-light">
      <div class="col">{{ title }}</div>
    </div>
    <div class="content column col">
      <slot></slot>
    </div>
  </div>
</template>

<script setup>
defineProps({
  title: {
    Type: String,
    default: null
  }
})
</script>

<style lang="sass" scoped>
.widget-pane
  overflow: hidden
  background-color: $amina-background-secondary-color
  background-radius: 8px
  border-radius: 8px
  border-color: $amina-separator-color
  border-width: 1px
  border-style: solid
  box-shadow: rgba(0, 0, 0, 0.24) 0px 0px 18px

  .header
    height: $amina-control-height
    display: flex
    justify-content: left
    align-items: center
    padding-left: $amina-default-padding
    background-color: $amina-primary-color
    border-color: $amina-separator-color
    border-radius: 8px 8px 0 0
    border-width: 0 0 1px 0
    border-style: solid

</style>
