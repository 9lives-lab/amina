import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event';

export class AminaTauriClient {
  constructor() {
    this.eventHandlers = {}

    listen('amina-event', (event) => {
      this.onMessage(event)
    })
  }

  onMessage(event) {
    const key = event.payload.key
    const data = JSON.parse(event.payload.data)

    if (key in this.eventHandlers) {
      const handlers = this.eventHandlers[key]
      for (const owner in handlers) {
        const handler = handlers[owner]
        handler(data)
      }
    }
  }

  setEventHandler (key, owner, handler) {
    if (!(key in this.eventHandlers)) {
      this.eventHandlers[key] = {}
    }
    this.eventHandlers[key][owner] = handler
  }

  removeEventHandler (key, owner) {
    if (key in this.eventHandlers) {
      delete this.eventHandlers[key][owner];
    }
  }

  async sendRequest (key, inputValue = { value: 0 }) {
    const request = JSON.stringify(inputValue)
    const res = await invoke("rpc_handler", { key: key, request: request })

    const response = JSON.parse(res)

    if (typeof response === 'object') {
      if ('Ok' in response) {
        return response.Ok
      } else if ('Err' in response.data) {
        throw response.Err
      } else {
        return response
      }
    } else {
      return response
    }
  }

  getFileUrl (keyPath) {
    return `${this.rpcAddress}get_file/${keyPath}`
  }

  handleCmd (name, args) {
    const intArgs = {}
    const boolArgs = {}
    const stringArgs = {}

    for (const key in args) {
      const value = args[key]
      if (typeof value === 'string') {
        stringArgs[key] = value
      } else if (typeof value === 'number') {
        intArgs[key] = value
      } else if (typeof value === 'boolean') {
        boolArgs[key] = value
      }
    }

    const callArgs = {
      cmd_name: name,
      args: {
        u64_list: intArgs,
        bool_list: boolArgs,
        string_list: stringArgs
      }
    }

    this.sendRequest('amina.cmd_manager.handle', callArgs)
  }
}
