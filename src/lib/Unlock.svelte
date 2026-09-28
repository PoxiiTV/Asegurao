<script lang="ts">
  import { onMount } from 'svelte';
  import { fly } from 'svelte/transition';
  import { listen } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { api } from './api';
  import AppIcon from './AppIcon.svelte';

  interface Prompt {
    exe: string;
    display: string;
    icon: string;
    auth_kind: string;
  }

  let prompt = $state<Prompt | null>(null);
  let secret = $state('');
  let shake = $state(false);
  let busy = $state(false);
  let input = $state<HTMLInputElement | null>(null);

  const win = getCurrentWindow();

  onMount(() => {
    const un = listen<Prompt>('lock:prompt', (e) => {
      prompt = e.payload;
      secret = '';
      setTimeout(() => input?.focus(), 60);
    });
    return () => {
      un.then((f) => f());
    };
  });

  async function submit() {
    if (!prompt || !secret || busy) return;
    busy = true;
    try {
      const res = await api.unlockAttempt(prompt.exe, secret);
      if (res === 'ok') {
        await win.hide();
        prompt = null;
      } else {
        shake = true;
        secret = '';
        setTimeout(() => (shake = false), 420);
        input?.focus();
      }
    } finally {
      busy = false;
    }
  }

  async function cancel() {
    if (!prompt) return;
    await api.unlockCancel(prompt.exe);
    await win.hide();
    prompt = null;
  }
</script>

{#if prompt}
  <div class="card" class:shake in:fly={{ y: 14, duration: 260 }}>
    <div class="drag" data-tauri-drag-region></div>
    <div class="ico-wrap">
      <AppIcon icon={prompt.icon} name={prompt.display} big />
      <span class="badge">🔒</span>
    </div>
    <h1>{prompt.display} está protegida</h1>
    <p class="sub">
      Introduce {prompt.auth_kind === 'master' ? 'la contraseña maestra' : 'la contraseña de la app'} para continuar
    </p>
    <input
      bind:this={input}
      class="field"
      type="password"
      placeholder="Contraseña"
      bind:value={secret}
      onkeydown={(e) => e.key === 'Enter' && submit()}
    />
    <button class="btn" disabled={busy || !secret} onclick={submit}>Desbloquear</button>
    <button class="ghost-link" onclick={cancel}>Cancelar y cerrar</button>
  </div>
{/if}

<style>
  .card {
    width: 340px;
    margin: 14px auto;
    padding: 30px 26px 22px;
    border-radius: 22px;
    text-align: center;
    background: var(--app-bg);
    background-attachment: fixed;
    border: 1px solid var(--border2);
    display: flex;
    flex-direction: column;
    gap: 13px;
    position: relative;
  }
  .drag {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 34px;
    cursor: grab;
  }
  .ico-wrap {
    position: relative;
    width: fit-content;
    margin: 4px auto 2px;
  }
  .badge {
    position: absolute;
    right: -6px;
    bottom: -6px;
    width: 26px;
    height: 26px;
    border-radius: 50%;
    background: var(--accent);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
    border: 3px solid var(--surface);
  }
  h1 {
    font-size: 18px;
    font-weight: 700;
  }
  .sub {
    font-size: 13px;
    color: var(--muted);
    line-height: 1.4;
    margin-top: -4px;
  }
  .ghost-link {
    background: none;
    border: 0;
    color: var(--muted);
    font-size: 12px;
  }
  .ghost-link:hover {
    color: var(--text);
  }
</style>
