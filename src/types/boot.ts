export type BootEntryType =
  | 'windows'
  | 'linux'
  | 'removable'
  | 'network'
  | 'firmware'
  | 'unknown'

export interface BootEntry {
  id: string
  name: string
  description: string | null
  path: string | null
  device: string | null
  active: boolean
  current: boolean
  next: boolean
  order: number | null
  entryType: BootEntryType
  suspectedInvalid: boolean
}

export interface BootInfo {
  currentOs: string
  uefi: boolean
  secureBoot: boolean | null
  bootCurrent: string | null
  bootNext: string | null
  bootOrder: string[]
  entries: BootEntry[]
}
