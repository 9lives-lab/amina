<template>
  <q-item :to="link">
    <q-item-section v-if="icon" avatar>
      <q-icon class="router-icon" :name="icon" :style="color_style" />
    </q-item-section>
  </q-item>
</template>

<script setup>
defineProps({
  title: {
    type: String,
    required: true
  },
  link: {
    type: String,
    default: '#'
  },
  icon: {
    type: String,
    default: ''
  },
  color_style: {
    type: String,
    default: ''
  }
})
</script>

<style lang="sass" scoped>
.router-icon
  color: #fff

</style>
