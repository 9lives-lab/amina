<template>
  <TitledPane :title="titleValue">
    <template #header>
      <div class="row justify-end">
        <q-btn class="" label="Run" type="submit" color="primary" icon="play_arrow"/>
      </div>
    </template>
    <template #default>
      <div class="column q-pa-md q-gutter-md">
        <CommandArgPane v-for="item in argsDescription" :key="item.call_name" :argDescription="item" />
      </div>
    </template>
  </TitledPane>
</template>

<script setup>
import { getCurrentInstance, ref, watch } from 'vue'
import TitledPane from 'amina_ui/components/TitledPane.vue'
import CommandArgPane from 'amina_ui/components/commands/CommandArgPane.vue'

const aminaApi = getCurrentInstance().appContext.config.globalProperties.$aminaApi

const props = defineProps({
  cmdName: String
})

const titleValue = ref('Run')
const argsDescription = ref({})

watch(() => props.cmdName, async (newCmdName) => {
  titleValue.value = 'Run: ' + newCmdName
  const resp = await aminaApi.sendRequest('amina.cmd_manager.get_command_description', { cmd_name: newCmdName })
  argsDescription.value = resp.args
})
</script>

<style lang="sass" scoped>

</style>
