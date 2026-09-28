<script lang="ts">
  let { icon = '', name = '?', big = false }: { icon?: string; name?: string; big?: boolean } =
    $props();

  const colors = ['#5865f2', '#1db954', '#e4405f', '#ff7a00', '#00b8d4', '#7b2ff7', '#ef4444', '#0ea5e9'];
  const letter = $derived((name.trim()[0] || '?').toUpperCase());
  const color = $derived(
    colors[[...name].reduce((a, c) => a + c.charCodeAt(0), 0) % colors.length]
  );
</script>

<div class="ico" class:big style={icon ? '' : `background:${color}`}>
  {#if icon}
    <img src={`data:image/png;base64,${icon}`} alt={name} />
  {:else}
    {letter}
  {/if}
</div>

<style>
  .ico {
    width: 38px;
    height: 38px;
    border-radius: 11px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 800;
    font-size: 16px;
    color: #fff;
    flex: none;
    overflow: hidden;
  }
  .ico.big {
    width: 50px;
    height: 50px;
    border-radius: 14px;
    font-size: 22px;
  }
  img {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }
</style>
