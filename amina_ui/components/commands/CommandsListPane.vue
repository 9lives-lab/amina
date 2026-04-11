<template>
  <q-tree
    :nodes="commandsList"
    default-expand-all
    v-model:selected="selected"
    node-key="label"
    dense
  />
</template>

<script setup>
import { getCurrentInstance, ref, watch, onMounted } from 'vue'

const emit = defineEmits(['commandSelected'])
const aminaApi = getCurrentInstance().appContext.config.globalProperties.$aminaApi
const selected = ref(null)
const commandsList = ref([])

async function update () {
  const resp = await aminaApi.sendRequest('amina.cmd_manager.get_commands_description')
  commandsList.value = resp.command_names.map(name => ({ label: name }))
}

watch(selected, async (newSelected) => {
  emit('commandSelected', newSelected)
})

onMounted(async () => {
  await update()
})
</script>

<style lang="sass" scoped>
.commands-pane
  height: 100%
</style>
