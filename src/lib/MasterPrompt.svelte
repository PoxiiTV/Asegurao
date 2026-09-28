<script lang="ts">
  import Modal from './Modal.svelte';
  import { masterReq, resolveMaster } from './prompt.svelte';

  let secret = $state('');

  function confirm() {
    if (!secret) return;
    const s = secret;
    secret = '';
    resolveMaster(s);
  }
  function cancel() {
    secret = '';
    resolveMaster(null);
  }
</script>

{#if masterReq.open}
  <Modal title="Confirmar con la contraseña maestra" onclose={cancel}>
    <p class="note">
      {masterReq.note || 'Esta acción reduce la protección.'} Después, Windows te pedirá también tu
      contraseña.
    </p>
    <input
      class="field"
      type="password"
      placeholder="Contraseña maestra"
      bind:value={secret}
      onkeydown={(e) => e.key === 'Enter' && confirm()}
      autofocus
    />
    <div class="row">
      <button class="btn ghost" onclick={cancel}>Cancelar</button>
      <button class="btn" onclick={confirm} disabled={!secret}>Confirmar</button>
    </div>
  </Modal>
{/if}

<style>
  .note {
    font-size: 13px;
    color: var(--muted);
    line-height: 1.5;
    margin-bottom: 14px;
  }
  .row {
    display: flex;
    gap: 10px;
    margin-top: 14px;
  }
  .row .btn {
    flex: 1;
  }
</style>
