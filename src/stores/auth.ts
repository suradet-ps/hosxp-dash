import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

/** Sanitized HOSxP officer returned by the backend after login. */
export interface SessionUser {
  officerId: number
  loginName: string
  displayName: string
  doctorCode: string | null
  role: string
}

/**
 * Officer session state.
 *
 * The backend owns the session; this store only mirrors it for the UI. A
 * restart, logout, or changed connection clears the backend session and the
 * next `restoreSession()` returns null.
 */
export const useAuthStore = defineStore('auth', () => {
  const session = ref<SessionUser | null>(null)
  const busy = ref(false)
  const error = ref<string | null>(null)

  const initials = computed(() => {
    const user = session.value
    if (!user) return ''
    const source = user.displayName.trim() || user.loginName.trim()
    return source ? source.charAt(0).toUpperCase() : ''
  })

  /** Verifies the credentials in the backend and mirrors the session. */
  async function login(username: string, password: string): Promise<boolean> {
    busy.value = true
    error.value = null
    try {
      session.value = await invoke<SessionUser>('login', {
        input: { username: username.trim(), password },
      })
      return true
    } catch (e) {
      session.value = null
      error.value = String(e)
      return false
    } finally {
      busy.value = false
    }
  }

  /** Clears the backend session and the mirrored state. */
  async function logout() {
    try {
      await invoke('logout')
    } catch {
      // The local session is cleared regardless; the app is unusable without
      // the login gate, so a failed IPC call must not keep the operator in.
    }
    session.value = null
    error.value = null
  }

  /** Mirrors the backend session (startup and after a connection change). */
  async function restoreSession() {
    try {
      session.value = await invoke<SessionUser | null>('current_session')
    } catch {
      session.value = null
    }
  }

  return {
    session,
    busy,
    error,
    initials,
    login,
    logout,
    restoreSession,
  }
})
