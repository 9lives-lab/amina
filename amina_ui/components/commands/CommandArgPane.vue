<template>
  <div class="row items-center">
    <q-checkbox
      v-if="argDescription.arg_type === 'BOOL'"
      class="col-3 q-mr-md"
      v-model="boolValue"
      :label="argDescription.call_name"
      :dense="true"
    />
    <q-input
      v-else
      class="col-3 q-mr-md"
      standout
      v-model="text"
      :label="argDescription.call_name"
      :dense="true"
    />
    <span class="col">{{ argDescription.description }}</span>
  </div>
</template>

<script setup>
import { ref } from 'vue'

defineProps({
  argDescription: Object
})

const boolValue = ref(false)
const text = ref('Run')

</script>

<style lang="sass" scoped>

</style>
