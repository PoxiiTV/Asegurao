<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { fly } from 'svelte/transition';

  let secret = $state('');
  let shake = $state(false);
  let busy = $state(false);
  let input = $state<HTMLInputElement | null>(null);

  async function submit() {
    if (!secret || busy) return;
    busy = true;
    // Si es correcta, el proceso termina con código 0 (no vuelve de aquí).
    const ok = await invoke<boolean>('gate_check', { secret });
    busy = false;
    if (!ok) {
      shake = true;
      secret = '';
      setTimeout(() => (shake = false), 420);
      input?.focus();
    }
  }

  function cancel() {
    invoke('gate_cancel');
  }
</script>

<div class="wrap">
  <div class="card" class:shake in:fly={{ y: 14, duration: 260 }}>
    <div class="ico">🛡️<span class="badge">🔒</span></div>
    <h1>Desinstalar Asegurao</h1>
    <p class="sub">Introduce tu contraseña maestra para confirmar la desinstalación.</p>
    <input
      bind:this={input}
      class="field"
      type="password"
      placeholder="Contraseña maestra"
      bind:value={secret}
      onkeydown={(e) => e.key === 'Enter' && submit()}
      autofocus
    />
    <button class="btn danger" disabled={busy || !secret} onclick={submit}>Desinstalar</button>
    <button class="ghost-link" onclick={cancel}>Cancelar</button>
  </div>
</div>

<style>
  .wrap {
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 20px;
  }
  .card {
    width: 100%;
    max-width: 340px;
    padding: 30px 26px 22px;
    border-radius: 20px;
    text-align: center;
    background: var(--surface);
    border: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 13px;
  }
  .ico {
    font-size: 40px;
    position: relative;
    width: fit-content;
    margin: 0 auto;
  }
  .badge {
    position: absolute;
    right: -8px;
    bottom: -2px;
    font-size: 18px;
  }
  h1 {
    font-size: 20px;
    font-weight: 800;
  }
  .sub {
    font-size: 13px;
    color: var(--muted);
    line-height: 1.45;
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
