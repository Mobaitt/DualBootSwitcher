import { invoke } from '@tauri-apps/api/core'
import { ref } from 'vue'
import type { BootEntry, BootInfo } from '../types/boot'

export function useBootManager() {
  const info = ref<BootInfo | null>(null)
  const loading = ref(false)
  const error = ref('')

  async function refresh() {
    loading.value = true
    error.value = ''
    try {
      info.value = await invoke<BootInfo>('refresh_boot_info')
    } catch (err) {
      error.value = String(err)
    } finally {
      loading.value = false
    }
  }

  async function setNext(id: string) {
    return invoke<void>('set_boot_next', { id })
  }

  async function setNextAndReboot(id: string) {
    return invoke<void>('set_boot_next_and_reboot', { id })
  }

  async function setDefaultBoot(id: string) {
    return invoke<void>('set_default_boot', { id })
  }

  async function reboot() {
    return invoke<void>('reboot')
  }

  const entries = () => info.value?.entries ?? ([] as BootEntry[])

  return { info, entries, loading, error, refresh, setNext, setNextAndReboot, setDefaultBoot, reboot }
}
