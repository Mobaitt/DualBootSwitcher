export type AppLanguage = 'zh-CN' | 'en-US'

export const translations = {
  'zh-CN': {
    appName: '引导序', windowTitle: '引导序',
    tagline: '下一次启动，由你选择。',
    themeSystem: '跟随系统', themeLight: '浅色', themeDark: '深色',
    refresh: '刷新', scanning: '扫描中', refreshing: '正在刷新…',
    readFailed: '读取失败', operationFailed: '操作未完成', completed: '已完成',
    nextBoot: '下一次启动', nextBootWillApply: '下次重启时生效', noBootNext: '按默认顺序启动',
    bootNext: 'BootNext', ready: '已就绪', default: '默认',
    currentSystem: '当前系统', uefi: 'UEFI', legacy: 'Legacy BIOS',
    defaultOrder: '默认顺序', noOrder: '未读取到启动顺序',
    availableSystems: '可用系统', selectHint: '选择系统，设置启动方式',
    reading: '正在读取启动项…', noEntries: '没有可用的 UEFI 启动项。',
    windows: 'Windows', linux: 'Linux', uefiType: 'UEFI',
    nextBootTag: '下一次启动', currentTag: '当前系统', defaultTag: '默认',
    selectEntry: '选择启动项', noEfiPath: '未提供 EFI 路径', cancel: '取消',
    setNext: '设为下一次启动', setDefault: '设为默认', setReboot: '设置并重启',
    nextSuccess: '下一次启动将进入「{name}」。', defaultSuccess: '已将「{name}」设为默认启动项。',
    trayNextSuccess: '下一次启动将进入「{name}」。', trayShow: '显示应用', trayHide: '隐藏应用',
    trayRefresh: '刷新启动项', trayQuit: '关闭应用', trayNextMenu: '设置下一次启动',
    trayNoEntries: '没有可用启动项',
  },
  'en-US': {
    appName: 'BootPilot', windowTitle: 'BootPilot',
    tagline: 'Your next boot. Your choice.',
    themeSystem: 'System', themeLight: 'Light', themeDark: 'Dark',
    refresh: 'Refresh', scanning: 'Scanning', refreshing: 'Refreshing…',
    readFailed: 'Read failed', operationFailed: 'Operation failed', completed: 'Done',
    nextBoot: 'Next boot', nextBootWillApply: 'Applied on next restart', noBootNext: 'Using default order',
    bootNext: 'BootNext', ready: 'Ready', default: 'Default',
    currentSystem: 'Current system', uefi: 'UEFI', legacy: 'Legacy BIOS',
    defaultOrder: 'Default order', noOrder: 'Boot order unavailable',
    availableSystems: 'Available systems', selectHint: 'Choose a system and set how it starts',
    reading: 'Reading boot entries…', noEntries: 'No usable UEFI boot entries found.',
    windows: 'Windows', linux: 'Linux', uefiType: 'UEFI',
    nextBootTag: 'Next boot', currentTag: 'Current', defaultTag: 'Default',
    selectEntry: 'Select boot entry', noEfiPath: 'EFI path unavailable', cancel: 'Cancel',
    setNext: 'Set next boot', setDefault: 'Set as default', setReboot: 'Set and restart',
    nextSuccess: 'The next boot will start “{name}”.', defaultSuccess: '“{name}” is now the default boot entry.',
    trayNextSuccess: 'The next boot will start “{name}”.', trayShow: 'Show app', trayHide: 'Hide app',
    trayRefresh: 'Refresh boot entries', trayQuit: 'Quit app', trayNextMenu: 'Set next boot',
    trayNoEntries: 'No usable boot entries',
  },
} as const

export type TranslationKey = keyof typeof translations['zh-CN']

export function detectLanguage(): AppLanguage {
  return navigator.language.toLowerCase().startsWith('zh') ? 'zh-CN' : 'en-US'
}
