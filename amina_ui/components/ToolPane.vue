<template>
  <div class="amina-tool-pane row">
    <slot></slot>
  </div>
</template>

<script setup>

</script>

<style lang="sass">
.amina-tool-pane
  background-color: $amina-background-primary-color
  border-style: solid
  border-width: 0 0 1px 0
  border-color: $amina-separator-color

  .bg-primary
    background-color: $amina-background-primary-color !important

  .q-btn
    border-radius: 0px

  .q-btn::before
      box-shadow: none

  .q-field
    .q-field__inner
      .q-field__control
        background-color: $amina-background-primary-color
        border-radius: 0px

  .q-field--outlined .q-field__control::before
    border-width: 0px

</style>
