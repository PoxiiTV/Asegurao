<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { listen } from '@tauri-apps/api/event';
  import { api, applyTheme, type Snapshot } from '$lib/api';
  import Setup from '$lib/Setup.svelte';
  import Login from '$lib/Login.svelte';
  import Main from '$lib/Main.svelte';
  import Unlock from '$lib/Unlock.svelte';
  import Gate from '$lib/Gate.svelte';
  import Toasts from '$lib/Toasts.svelte';

  const label = getCurrentWindow().label;
  const isUnlock = label === 'unlock';
  const isGate = label === 'gate';

  // La ventana flotante de desbloqueo es transparente: quitamos el fondo del body.
  if (isUnlock && typeof document !== 'undefined') {
    document.documentElement.style.background = 'transparent';
    document.body.style.background = 'transparent';
  }

  let stage = $state<'loading' | 'setup' | 'login' | 'main'>('loading');
  let snap = $state<Snapshot | null>(null);

  async function reload() {
    snap = await api.snapshot();
    applyTheme(snap.settings.theme);
    return snap;
  }

  async function boot() {
    const s = await reload();
    if (!s.configured) {
      stage = 'setup';
    } else {
      const active = await api.sessionStatus();
      stage = active ? 'main' : 'login';
    }
  }

  onMount(() => {
    if (isUnlock || isGate) return;
    boot();
    const un = listen('session:lock', () => {
      stage = 'login';
    });
    return () => un.then((f) => f());
  });
</script>

{#if isUnlock}
  <Unlock />
{:else if isGate}
  <Gate />
{:else}
  {#if stage === 'setup'}
    <Setup onDone={async () => { await reload(); stage = 'main'; }} />
  {:else if stage === 'login'}
    <Login authKind={snap?.auth_kind ?? 'pin'} onDone={async () => { await reload(); stage = 'main'; }} />
  {:else if stage === 'main' && snap}
    <Main {snap} refresh={reload} onLock={() => (stage = 'login')} />
  {/if}
  <Toasts />
{/if}
