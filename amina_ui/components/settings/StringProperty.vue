<template>
  <q-item clickable v-ripple >
    <q-item-section side>
      <q-item-label>{{ item.name }}</q-item-label>
      <q-item-label caption>{{ item.description }}</q-item-label>
    </q-item-section>
    <q-item-section side>
      <q-btn outline label="Save" @click="saveValue()" />
    </q-item-section>
    <q-item-section side>
      <q-btn outline icon="refresh" @click="refreshValue()" />
    </q-item-section>
    <q-item-section >
      <q-input outlined dense v-model="value" />
    </q-item-section>
  </q-item>
</template>

<script setup>
import { ref, getCurrentInstance, onMounted } from 'vue'

const aminaApi = getCurrentInstance().appContext.config.globalProperties.$aminaApi
const value = ref('')

const props = defineProps({
  item: Object
})

async function refreshValue () {
  const newValue = await aminaApi.sendRequest('amina_core.settings_manager.get_string_value', { key: props.item.name })
  value.value = newValue
}

async function saveValue () {
  await aminaApi.sendRequest('amina_core.settings_manager.set_string_value', { key: props.item.name, data: value.value })
  await refreshValue()
}

onMounted(async () => {
  await refreshValue()
})

</script>

<style lang="sass" scoped>

</style>
