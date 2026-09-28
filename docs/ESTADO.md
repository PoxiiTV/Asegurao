# Asegurao — Estado del proyecto

Bloqueador de apps de Windows con contraseña. Reescritura en español de
[AppLocker](https://github.com/Hachiiki/AppLocker). **Stack:** Tauri 2 · Rust ·
Svelte 5 · Vite. Idioma: solo español. Repo público: `PoxiiTV/Asegurao` (release v1.0.0 con el setup).

## Arquitectura

**Rust** (`src-tauri/src/`):
- `store.rs` — config.json en `%APPDATA%\Asegurao`, hashes Argon2id, escritura atómica.
- `procctl.rs` — procesos Windows: listar (solo sesión del usuario; ignora servicios de sesión 0), suspender/reanudar (NtSuspendProcess), cerrar, icono real del .exe → PNG.
- `apps.rs` — descubrir apps (registro Uninstall + procesos abiertos).
- `watcher.rs` — hilo cada 250 ms: congela app bloqueada y abre ventana de desbloqueo; auto-bloqueo de sesión.
  Solo un desbloqueo `pending` a la vez: mientras exista, no vigila nada más. La ventana `unlock`
  recupera el aviso al montarse (`unlock_pending`) por si el evento llegó antes de cargar.
- `state.rs` — estado en memoria (Instant, a prueba de cambio de hora).
- `history.rs` — historial (máx. 1000, rota) + carpeta `fotos/`.
- `intruder.rs` — captura webcam opcional (nokhwa) tras N fallos.
- `uac.rs` — 2ª llave: `ShellExecuteW runas` (confirmación de Windows).
- `commands.rs` — comandos Tauri. `lib.rs` — plugins, bandeja, ventanas, modo gate.

**Frontend** (`src/`): `+page.svelte` conmuta por etiqueta de ventana
(`main` / `unlock` / `gate`). Componentes en `src/lib/`. Temas por variables CSS
en `app.css` (Grafito por defecto, Bóveda, Aurora).

**Ventanas:** `main` (asistente/login/panel), `unlock` (tarjeta flotante
transparente, `shadow:false`), `gate` (contraseña al desinstalar).

## Decisiones clave

- **Congela** (suspende) la app en vez de cerrarla; reanuda al acertar.
- **Reforzar** protección = solo sesión. **Debilitar** (quitar app, bajar nivel,
  borrar historial, cambiar maestra, desinstalar) = maestra **+ UAC**.
- **Nivel Fortaleza** por defecto. PIN mín. 4 dígitos / contraseña mín. 8.
- **Sin driver de kernel** y **sin anti-tamper** (rootkit): descartado por decisión
  de diseño (comportamiento malware + bloquearía al propio dueño). El techo real
  contra un admin es BitLocker + BIOS (fuera de la app).
- **Desinstalar** pide la maestra vía hook NSIS `PREUNINSTALL` → `--uninstall-gate`.

## Verificado

Compila + tests Rust OK. Probado por el usuario en su PC: bloquea OK.
`deploy.bat` genera setup NSIS (~1.6 MB) + portable (~4.4 MB) en `deploy-hosting/`.
Release en GitHub: solo el setup.

## Pendiente / v2

- Arranque **elevado y silencioso** con Windows (tarea programada) — hoy autostart
  normal; congelar apps del mismo usuario funciona sin admin, apps elevadas no.
- `Allow::UntilExit` con apps que viven en bandeja (Telegram): cerrar la ventana no las
  cierra, siguen desbloqueadas hasta salir desde la bandeja.
- Modo Extremo / Fortaleza avanzado (bloqueo de herramientas): descartado por ahora.
- Landing web pendiente.
