<script lang="ts">
  import { onMount } from 'svelte';
  import { fly } from 'svelte/transition';
  import { api, type HistoryEntry } from './api';
  import { guard } from './toast.svelte';
  import { askMaster } from './prompt.svelte';
  import AppIcon from './AppIcon.svelte';

  let entries = $state<HistoryEntry[]>([]);
  let filter = $state<'todo' | 'denied' | 'tamper'>('todo');

  const filtered = $derived(
    filter === 'todo' ? entries : entries.filter((e) => e.result === filter)
  );

  async function load() {
    entries = await api.getHistory();
  }

  async function clear() {
    const m = await askMaster('Vas a borrar todo el historial.');
    if (m === null) return;
    const ok = await guard(() => api.clearHistory(m), 'Historial borrado');
    if (ok) load();
  }

  function when(ts: number) {
    const d = new Date(ts * 1000);
    return d.toLocaleString('es-ES', {
      day: '2-digit',
      month: 'short',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  const meta: Record<string, { label: string; cls: string }> = {
    allowed: { label: 'Acceso concedido', cls: 'ok' },
    denied: { label: 'Acceso denegado', cls: 'bad' },
    tamper: { label: 'Intento de manipulación', cls: 'warn' }
  };

  onMount(load);
</script>

<div class="head">
  <div>
    <h2>Historial</h2>
    <p class="sub">Quién intentó abrir qué y cuándo</p>
  </div>
  {#if entries.length}
    <button class="btn ghost sm" onclick={clear}>Borrar historial</button>
  {/if}
</div>

<div class="tabs">
  <button class:on={filter === 'todo'} onclick={() => (filter = 'todo')}>Todo</button>
  <button class:on={filter === 'denied'} onclick={() => (filter = 'denied')}>Denegados</button>
  <button class:on={filter === 'tamper'} onclick={() => (filter = 'tamper')}>Manipulación</button>
</div>

{#if filtered.length === 0}
  <div class="empty">
    <div class="e-ico">📭</div>
    <p>No hay registros{filter !== 'todo' ? ' de este tipo' : ' todavía'}</p>
  </div>
{:else}
  <div class="list">
    {#each filtered as e, i (e.ts + e.exe + i)}
      <div class="item" in:fly={{ y: 8, duration: 200, delay: Math.min(i * 20, 200) }}>
        <AppIcon icon="" name={e.display} />
        <div class="meta">
          <span class="t">{e.display}</span>
          <span class="s {meta[e.result]?.cls}">{meta[e.result]?.label ?? e.result}</span>
        </div>
        {#if e.photo}
          <div class="photo" title="Foto del intruso">📷</div>
        {/if}
        <span class="time">{when(e.ts)}</span>
      </div>
    {/each}
  </div>
{/if}

<style>
  .head {
    display: flex;
    align-items: center;
    margin-bottom: 16px;
  }
  .head h2 {
    font-size: 22px;
    font-weight: 800;
  }
  .sub {
    color: var(--muted);
    font-size: 13px;
    margin-top: 2px;
  }
  .head .btn {
    margin-left: auto;
  }
  .tabs {
    display: flex;
    gap: 4px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 4px;
    margin-bottom: 16px;
    width: fit-content;
  }
  .tabs button {
    padding: 8px 16px;
    border: 0;
    border-radius: 9px;
    background: transparent;
    color: var(--muted);
    font-weight: 600;
    font-size: 13px;
  }
  .tabs button.on {
    background: var(--surface3);
    color: var(--text);
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 7px;
  }
  .item {
    display: flex;
    align-items: center;
    gap: 13px;
    padding: 10px 14px;
    border-radius: 13px;
    background: var(--surface);
    border: 1px solid var(--border);
  }
  .meta {
    display: flex;
    flex-direction: column;
    flex: 1;
  }
  .meta .t {
    font-size: 14px;
    font-weight: 600;
  }
  .meta .s {
    font-size: 12px;
  }
  .s.ok {
    color: var(--ok);
  }
  .s.bad {
    color: var(--danger);
  }
  .s.warn {
    color: var(--accent);
  }
  .photo {
    font-size: 16px;
  }
  .time {
    font-size: 12px;
    color: var(--muted);
    white-space: nowrap;
  }
  .empty {
    text-align: center;
    padding: 60px 20px;
    color: var(--muted);
  }
  .e-ico {
    font-size: 46px;
    margin-bottom: 8px;
  }
</style>
