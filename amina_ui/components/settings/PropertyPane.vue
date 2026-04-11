<template>
  <div v-if="item.type === 'header'">
    <q-separator spaced v-if="!item.isFirstSection"/>
    <q-item-label header>{{ item.text }}</q-item-label>
  </div>
  <StringProperty v-if="item.type === 'string_property'" v-bind:item="item" />
</template>

<script setup>
import StringProperty from 'amina_ui/components/settings/StringProperty.vue'

defineProps({
  item: Object
})
</script>

<style lang="sass" scoped>

</style>
