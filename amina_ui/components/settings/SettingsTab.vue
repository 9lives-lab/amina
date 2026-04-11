<template>
  <WidgetPane title="General">
    <q-list padding>
      <PropertyPane v-for="item of propsList" v-bind:item="item" v-bind:key="item.name" />
    </q-list>
  </WidgetPane>
</template>

<script setup>
import { ref, getCurrentInstance } from 'vue'
import WidgetPane from 'amina_ui/components/WidgetPane.vue'
import PropertyPane from 'amina_ui/components/settings/PropertyPane.vue'

const aminaApi = getCurrentInstance().appContext.config.globalProperties.$aminaApi
const propsList = ref([])

async function updateTab (tabName) {
  const tabDescription = await aminaApi.sendRequest('amina_core.settings_manager.get_tab', { tab_name: tabName })
  let isFirstSection = true
  let index = 0

  const newPropsList = tabDescription.sections.reduce((acc, section) => {
    acc.push({
      idx: index++,
      type: 'header',
      text: section.name,
      isFirstSection
    })
    isFirstSection = false

    const properties = section.properties.map(prop => ({
      idx: index++,
      type: 'string_property',
      name: prop.name,
      description: 'No description'
    }))

    return acc.concat(properties)
  }, [])

  propsList.value = newPropsList
}

defineExpose({
  updateTab
})

</script>

<style lang="sass" scoped>
.name-field
  text-align: center
</style>
