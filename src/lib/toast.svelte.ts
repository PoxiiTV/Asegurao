export interface Toast {
  id: number;
  msg: string;
  kind: 'info' | 'ok' | 'error';
}

let seq = 0;
export const toasts = $state<Toast[]>([]);

export function toast(msg: string, kind: Toast['kind'] = 'info') {
  const id = ++seq;
  toasts.push({ id, msg, kind });
  setTimeout(() => {
    const i = toasts.findIndex((t) => t.id === id);
    if (i >= 0) toasts.splice(i, 1);
  }, 3200);
}

/** Ejecuta una acción async y muestra el error si falla. */
export async function guard(fn: () => Promise<unknown>, okMsg?: string) {
  try {
    await fn();
    if (okMsg) toast(okMsg, 'ok');
    return true;
  } catch (e) {
    toast(String(e), 'error');
    return false;
  }
}
