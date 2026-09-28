<script lang="ts">
  import { fly } from 'svelte/transition';
  import { api, type AuthKind } from './api';
  import { toast } from './toast.svelte';
  import Modal from './Modal.svelte';

  let { authKind, onDone }: { authKind: string; onDone: () => void } = $props();

  let secret = $state('');
  let busy = $state(false);
  let shake = $state(false);
  let showRecover = $state(false);

  // Recuperación
  let rCode = $state('');
  let rKind = $state<AuthKind>('pin');
  let rSecret = $state('');
  let newCode = $state('');

  async function submit() {
    if (!secret || busy) return;
    busy = true;
    try {
      const ok = await api.login(secret);
      if (ok) {
        onDone();
      } else {
        wrong();
      }
    } catch (e) {
      toast(String(e), 'error');
    }
    busy = false;
  }

  function wrong() {
    shake = true;
    secret = '';
    setTimeout(() => (shake = false), 420);
    toast('Contraseña incorrecta', 'error');
  }

  async function doRecover() {
    if (!rCode || rSecret.length < (rKind === 'pin' ? 4 : 8)) {
      toast('Revisa el código y la nueva contraseña', 'error');
      return;
    }
    try {
      newCode = await api.recover(rCode, rKind, rSecret);
      toast('Contraseña restablecida', 'ok');
    } catch (e) {
      toast(String(e), 'error');
    }
  }
</script>

<div class="screen">
  <div class="card lock" class:shake in:fly={{ y: 12, duration: 300 }}>
    <div class="logo">🛡️</div>
    <h1>Asegurao</h1>
    <p class="sub">Introduce tu {authKind === 'pin' ? 'PIN' : 'contraseña'} para entrar</p>
    <input
      class="field"
      type="password"
      inputmode={authKind === 'pin' ? 'numeric' : 'text'}
      placeholder={authKind === 'pin' ? 'PIN' : 'Contraseña'}
      bind:value={secret}
      onkeydown={(e) => e.key === 'Enter' && submit()}
      autofocus
    />
    <button class="btn" disabled={busy || !secret} onclick={submit}>Entrar</button>
    <button class="link" onclick={() => (showRecover = true)}>He olvidado mi contraseña</button>
  </div>
</div>

{#if showRecover}
  <Modal title="Recuperar acceso" onclose={() => (showRecover = false)}>
    {#if newCode}
      <p class="rsub">Listo. Guarda tu nuevo código de recuperación:</p>
      <div class="ncode">{newCode}</div>
      <button class="btn" style="width:100%;margin-top:14px" onclick={() => { showRecover = false; newCode=''; }}>
        Entendido
      </button>
    {:else}
      <p class="rsub">Introduce tu código de recuperación y elige una nueva contraseña maestra.</p>
      <input class="field" placeholder="Código de recuperación" bind:value={rCode} style="margin-bottom:10px" />
      <div class="seg">
        <button class:on={rKind === 'pin'} onclick={() => (rKind = 'pin')}>PIN</button>
        <button class:on={rKind === 'password'} onclick={() => (rKind = 'password')}>Contraseña</button>
      </div>
      <input
        class="field"
        type="password"
        placeholder={rKind === 'pin' ? 'Nuevo PIN (mín. 4)' : 'Nueva contraseña (mín. 8)'}
        bind:value={rSecret}
        style="margin-top:10px"
      />
      <button class="btn" style="width:100%;margin-top:14px" onclick={doRecover}>Restablecer</button>
    {/if}
  </Modal>
{/if}

<style>
  .screen {
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
  }
  .lock {
    width: 340px;
    max-width: 100%;
    padding: 32px 28px;
    text-align: center;
    display: flex;
    flex-direction: column;
    gap: 14px;
    background: var(--surface);
    backdrop-filter: blur(20px);
    box-shadow: var(--shadow);
  }
  .logo {
    font-size: 44px;
  }
  h1 {
    font-size: 22px;
    font-weight: 800;
  }
  .sub {
    color: var(--muted);
    font-size: 13px;
  }
  .link {
    background: none;
    border: 0;
    color: var(--muted);
    font-size: 12px;
    text-decoration: underline;
  }
  .link:hover {
    color: var(--text);
  }
  .rsub {
    color: var(--muted);
    font-size: 13px;
    line-height: 1.5;
    margin-bottom: 12px;
  }
  .ncode {
    font-family: 'Courier New', monospace;
    font-size: 20px;
    font-weight: 700;
    letter-spacing: 2px;
    padding: 16px;
    border-radius: 12px;
    background: var(--accent-soft);
    border: 1px dashed var(--accent);
    color: var(--accent);
    text-align: center;
  }
  .seg {
    display: flex;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 4px;
    gap: 4px;
  }
  .seg button {
    flex: 1;
    padding: 9px;
    border: 0;
    border-radius: 9px;
    background: transparent;
    color: var(--muted);
    font-weight: 600;
    font-size: 13px;
  }
  .seg button.on {
    background: var(--surface3);
    color: var(--text);
  }
</style>
