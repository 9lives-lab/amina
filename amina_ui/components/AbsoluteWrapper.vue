<template>
  <div class="amina-wrapper">
    <div class="amina-absolute-wrapper">
      <slot></slot>
    </div>
  </div>
</template>

<style lang="sass" scoped>

.amina-wrapper
  overflow: hidden
  position: relative

.amina-absolute-wrapper
  overflow: hidden
  position: absolute
  top: 0px
  bottom: 0px
  left: 0px
  right: 0px

</style>
