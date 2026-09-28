<script lang="ts">
  import { fly } from 'svelte/transition';
  import { api, type Snapshot } from './api';
  import Apps from './Apps.svelte';
  import History from './History.svelte';
  import Settings from './Settings.svelte';
  import MasterPrompt from './MasterPrompt.svelte';

  let { snap, refresh, onLock }: { snap: Snapshot; refresh: () => void; onLock: () => void } =
    $props();

  let view = $state<'apps' | 'historial' | 'ajustes'>('apps');

  const activeCount = $derived(snap.apps.filter((a) => a.enabled).length);

  let lastTouch = 0;
  function touch() {
    const now = Date.now();
    if (now - lastTouch > 5000) {
      lastTouch = now;
      api.touch();
    }
  }

  async function lock() {
    await api.lockSession();
    onLock();
  }
</script>

<svelte:window onpointerdown={touch} onkeydown={touch} />

<div class="shell">
  <aside>
    <div class="brand">
      <span class="mark">🛡️</span>
      Asegurao
    </div>

    <nav>
      <button class:on={view === 'apps'} onclick={() => (view = 'apps')}>
        <span class="ni">🗂️</span> Apps protegidas
      </button>
      <button class:on={view === 'historial'} onclick={() => (view = 'historial')}>
        <span class="ni">📜</span> Historial
      </button>
      <button class:on={view === 'ajustes'} onclick={() => (view = 'ajustes')}>
        <span class="ni">⚙️</span> Ajustes
      </button>
    </nav>

    <div class="status">
      <div class="s-top">
        <span class="dot"></span>
        {snap.settings.level === 'fortaleza' ? '🏰 Fortaleza' : '🔒 Protección'} activa
      </div>
      <p>{activeCount} {activeCount === 1 ? 'app vigilada' : 'apps vigiladas'}</p>
    </div>

    <button class="lock-btn" onclick={lock}>Bloquear sesión</button>
  </aside>

  <main>
    {#key view}
      <div class="content" in:fly={{ y: 10, duration: 220 }}>
        {#if view === 'apps'}
          <Apps apps={snap.apps} {refresh} />
        {:else if view === 'historial'}
          <History />
        {:else}
          <Settings settings={snap.settings} authKind={snap.auth_kind} {refresh} />
        {/if}
      </div>
    {/key}
  </main>
</div>

<MasterPrompt />

<style>
  .shell {
    display: flex;
    height: 100vh;
  }
  aside {
    width: 216px;
    flex: none;
    padding: 20px 16px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    background: rgba(0, 0, 0, 0.16);
    border-right: 1px solid var(--border);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 9px;
    font-weight: 800;
    font-size: 17px;
    padding: 4px 8px 20px;
  }
  .mark {
    font-size: 22px;
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  nav button {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 11px 12px;
    border: 0;
    border-radius: 11px;
    background: transparent;
    color: var(--muted);
    font-size: 14px;
    font-weight: 600;
    text-align: left;
    transition: background 0.12s ease, color 0.12s ease;
  }
  nav button:hover {
    background: var(--surface);
    color: var(--text);
  }
  nav button.on {
    background: var(--surface2);
    color: var(--text);
  }
  nav button.on .ni {
    filter: none;
  }
  .ni {
    font-size: 16px;
  }
  .status {
    margin-top: auto;
    padding: 13px;
    border-radius: 13px;
    background: var(--accent-soft);
    border: 1px solid color-mix(in srgb, var(--accent) 30%, transparent);
    font-size: 12px;
  }
  .s-top {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 700;
    color: var(--accent);
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--ok);
    box-shadow: 0 0 8px var(--ok);
  }
  .status p {
    color: var(--muted);
    margin-top: 4px;
  }
  .lock-btn {
    margin-top: 10px;
    padding: 10px;
    border-radius: 11px;
    background: var(--surface);
    border: 1px solid var(--border);
    color: var(--muted);
    font-weight: 600;
    font-size: 13px;
  }
  .lock-btn:hover {
    background: var(--surface2);
    color: var(--text);
  }
  main {
    flex: 1;
    overflow: auto;
    padding: 26px 30px;
  }
  .content {
    max-width: 760px;
  }
</style>
