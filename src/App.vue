<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { useBootManager } from './composables/useBootManager'
import type { BootEntry } from './types/boot'
import { detectLanguage, translations, type TranslationKey } from './i18n'

const { info, loading, error, refresh, setNext, setNextAndReboot, setDefaultBoot } = useBootManager()
const language = detectLanguage()
const copy = translations[language]
function text(key: TranslationKey) { return copy[key] }
function message(key: 'nextSuccess' | 'defaultSuccess', name: string) { return text(key).replace('{name}', name) }
function localizeError(raw: string) {
  if (language === 'zh-CN') return raw
  const known: Record<string, string> = {
    '当前平台暂不支持 UEFI 启动项管理': 'UEFI boot management is not supported on this platform.',
    '当前系统不是 UEFI 启动模式，BootNext 不可用': 'The current system is not booted in UEFI mode, so BootNext is unavailable.',
    '权限不足，请以管理员权限运行此操作': 'Permission denied. Run the app as administrator for this operation.',
    '找不到系统命令：efibootmgr': 'The efibootmgr command was not found.',
    '找不到系统命令：bcdedit': 'The bcdedit command was not found.',
  }
  return known[raw] || raw
}
const selected = ref<BootEntry | null>(null)
const actionError = ref('')
const actionSuccess = ref('')
type ThemeMode = 'system' | 'light' | 'dark'
const storedTheme = localStorage.getItem('dualboot-theme')
const themeMode = ref<ThemeMode>(storedTheme === 'light' || storedTheme === 'dark' ? storedTheme : 'system')
const systemThemeQuery = window.matchMedia('(prefers-color-scheme: dark)')
const systemDark = ref(systemThemeQuery.matches)
const dark = computed(() => themeMode.value === 'dark' || (themeMode.value === 'system' && systemDark.value))
const themeMenuOpen = ref(false)
const themeControl = ref<HTMLElement | null>(null)
const themeLabel = computed(() => themeMode.value === 'system' ? text('themeSystem') : themeMode.value === 'light' ? text('themeLight') : text('themeDark'))
let successTimer: ReturnType<typeof setTimeout> | undefined
let unlistenTrayResult: (() => void) | undefined

function isNext(entry: BootEntry) {
  return entry.next || entry.id.toLowerCase() === info.value?.bootNext?.toLowerCase()
}

const nextEntry = computed(() => info.value?.entries.find(isNext))

function orderLabel(id: string) {
  const direct = info.value?.entries.find((entry) => entry.id === id)?.name
  if (direct) return direct
  if (id === '{bootmgr}') return info.value?.entries.find((entry) => entry.entryType === 'windows')?.name || 'Windows Boot Manager'
  return id.startsWith('{') ? id : `Boot${id}`
}

function icon(entry: BootEntry) {
  if (entry.entryType === 'windows') return '⊞'
  if (entry.entryType === 'linux') return '◉'
  if (entry.entryType === 'removable' || entry.entryType === 'network') return '⌁'
  return '◌'
}

function openEntry(entry: BootEntry) {
  actionError.value = ''
  actionSuccess.value = ''
  selected.value = entry
}

function showSuccess(message: string) {
  actionSuccess.value = message
  if (successTimer) clearTimeout(successTimer)
  successTimer = setTimeout(() => { actionSuccess.value = ''; successTimer = undefined }, 3500)
}

function chooseThemeMode(mode: ThemeMode) {
  themeMode.value = mode
  localStorage.setItem('dualboot-theme', mode)
  themeMenuOpen.value = false
}

function syncSystemTheme(event: MediaQueryListEvent) { systemDark.value = event.matches }
function closeThemeMenuOnOutsideClick(event: PointerEvent) {
  if (themeControl.value && !themeControl.value.contains(event.target as Node)) themeMenuOpen.value = false
}

async function choose(action: 'next' | 'reboot' | 'default') {
  if (!selected.value) return
  actionError.value = ''
  actionSuccess.value = ''
  try {
    if (action === 'default') {
      await setDefaultBoot(selected.value.id)
      showSuccess(message('defaultSuccess', selected.value.name))
    } else if (action === 'next') {
      await setNext(selected.value.id)
      showSuccess(message('nextSuccess', selected.value.name))
    } else {
      await setNextAndReboot(selected.value.id)
      return
    }
    selected.value = null
    await refresh()
  } catch (err) { actionError.value = localizeError(String(err)) }
}

onMounted(async () => {
  document.title = text('windowTitle')
  await invoke('set_window_title', { title: text('windowTitle') }).catch(() => undefined)
  await invoke('set_tray_language', { language }).catch(() => undefined)
  systemThemeQuery.addEventListener('change', syncSystemTheme)
  document.addEventListener('pointerdown', closeThemeMenuOnOutsideClick)
  await refresh()
  unlistenTrayResult = await listen<{ ok: boolean; message: string }>('tray-boot-next-result', async ({ payload }) => {
    if (payload.ok) { actionError.value = ''; showSuccess(payload.message); await refresh() }
    else { actionSuccess.value = ''; actionError.value = localizeError(payload.message) }
  })
})
onUnmounted(() => {
  if (successTimer) clearTimeout(successTimer)
  unlistenTrayResult?.()
  systemThemeQuery.removeEventListener('change', syncSystemTheme)
  document.removeEventListener('pointerdown', closeThemeMenuOnOutsideClick)
})
</script>

<template>
  <main class="app-shell" :class="{ dark }">
    <header class="topbar">
      <p class="topbar-tagline">{{ text('tagline') }}</p>
      <div class="top-actions">
        <div ref="themeControl" class="theme-control"><button class="theme-trigger" type="button" :aria-expanded="themeMenuOpen" @click="themeMenuOpen = !themeMenuOpen"><span class="theme-symbol" aria-hidden="true"></span><strong>{{ themeLabel }}</strong><span class="theme-chevron" :class="{ open: themeMenuOpen }">⌄</span></button><Transition name="theme-menu"><div v-if="themeMenuOpen" class="theme-menu"><button v-for="mode in (['system', 'light', 'dark'] as ThemeMode[])" :key="mode" type="button" :class="{ selected: themeMode === mode }" @click="chooseThemeMode(mode)"><span class="theme-dot" :class="{ selected: themeMode === mode }"></span><span>{{ mode === 'system' ? text('themeSystem') : mode === 'light' ? text('themeLight') : text('themeDark') }}</span></button></div></Transition></div>
        <button class="refresh-button" :disabled="loading" @click="refresh"><span :class="{ spin: loading }">↻</span>{{ loading ? text('scanning') : text('refresh') }}</button>
      </div>
    </header>

    <div v-if="error" class="notice error"><strong>{{ text('readFailed') }}</strong><span>{{ localizeError(error) }}</span></div>
    <div v-if="actionError" class="notice error"><strong>{{ text('operationFailed') }}</strong><span>{{ actionError }}</span></div>
    <div v-if="actionSuccess" :key="actionSuccess" class="notice success"><strong>{{ text('completed') }}</strong><span>{{ actionSuccess }}</span></div>

    <section v-if="info" class="boot-focus">
      <div class="focus-main"><div class="focus-icon" aria-hidden="true"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><path d="M5 12h14m-6-6 6 6-6 6"/></svg></div><div class="focus-copy"><p>{{ text('nextBoot') }}</p><h1>{{ nextEntry?.name || text('noBootNext') }}</h1><span>{{ nextEntry ? text('nextBootWillApply') : text('noBootNext') }}</span></div><div class="focus-state"><i></i>{{ nextEntry ? text('ready') : text('default') }}</div></div>
      <div class="focus-details"><div class="system-detail"><span>{{ text('currentSystem') }}</span><strong>{{ info.currentOs }}</strong><em>{{ info.uefi ? text('uefi') : text('legacy') }}</em></div><div class="detail-divider"></div><div class="order-detail"><span>{{ text('defaultOrder') }}</span><div v-if="info.bootOrder.length" class="order-list"><span v-for="(id, index) in info.bootOrder" :key="id" class="order-item"><b>{{ index + 1 }}</b>{{ orderLabel(id) }}</span></div><strong v-else>{{ text('noOrder') }}</strong></div></div>
    </section>

    <section class="entries-section">
      <div class="section-heading"><h2>{{ text('availableSystems') }} <span>{{ info?.entries.length ?? 0 }}</span></h2><span class="scanning">{{ loading ? text('refreshing') : text('selectHint') }}</span></div>
      <div v-if="!info && loading" class="empty-state"><span class="loader"></span><p>{{ text('reading') }}</p></div>
      <TransitionGroup v-else-if="info?.entries.length" name="entry" tag="div" class="entry-list">
        <button v-for="entry in info.entries" :key="entry.id" class="entry-card" :class="{ current: entry.current, next: isNext(entry) }" @click="openEntry(entry)">
          <span class="entry-icon" :class="entry.entryType">{{ icon(entry) }}</span>
          <span class="entry-main"><strong>{{ entry.name }}</strong><span>{{ entry.entryType === 'windows' ? text('windows') : entry.entryType === 'linux' ? text('linux') : text('uefiType') }}</span></span>
          <span class="entry-meta"><span v-if="isNext(entry)" class="tag next-tag">{{ text('nextBootTag') }}</span><span v-else-if="entry.current" class="tag current-tag">{{ text('currentTag') }}</span><span v-if="entry.order !== null" class="order-tag">{{ text('defaultTag') }} #{{ entry.order + 1 }}</span></span><span class="chevron">›</span>
        </button>
      </TransitionGroup>
      <div v-else-if="info" class="empty-state"><p>{{ text('noEntries') }}</p></div>
    </section>

    <Transition name="modal"><div v-if="selected" class="modal-backdrop" @click.self="selected = null"><section class="modal"><button class="modal-close" @click="selected = null">×</button><p class="modal-kicker">{{ text('selectEntry') }}</p><h2>{{ selected.name }}</h2><p class="modal-path">{{ selected.path || text('noEfiPath') }}</p><div class="modal-actions"><button class="secondary" @click="selected = null">{{ text('cancel') }}</button><button class="secondary" @click="choose('next')">{{ text('setNext') }}</button><button class="permanent" @click="choose('default')">{{ text('setDefault') }}</button><button class="primary" @click="choose('reboot')">{{ text('setReboot') }}</button></div></section></div></Transition>
  </main>
</template>
