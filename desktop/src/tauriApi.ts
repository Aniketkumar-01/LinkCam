// Tauri v2 Command Invoker Helper

export async function invokeTauri<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
  if (typeof window !== 'undefined') {
    // Tauri v2 standard globals
    const tauriGlobal = (window as any).__TAURI__;
    if (tauriGlobal?.core?.invoke) {
      return tauriGlobal.core.invoke(cmd, args);
    }
    const tauriInternals = (window as any).__TAURI_INTERNALS__;
    if (tauriInternals?.invoke) {
      return tauriInternals.invoke(cmd, args);
    }
  }

  // Graceful fallback for browser dev / mock preview
  console.warn(`[Tauri API] "${cmd}" called outside of Tauri runtime.`);
  throw new Error(`Tauri runtime not found for command "${cmd}"`);
}
