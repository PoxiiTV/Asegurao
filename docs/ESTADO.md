# Asegurao — Estado del proyecto

Bloqueador de apps de Windows con contraseña. Reescritura en español de
[AppLocker](https://github.com/Hachiiki/AppLocker). **Stack:** Tauri 2 · Rust ·
Svelte 5 · Vite. Idioma: solo español. Repo público: `PoxiiTV/Asegurao`.
Versión actual: **1.1.0** (cambios en `CHANGELOG.md`).

## Arquitectura

**Rust** (`src-tauri/src/`):
- `store.rs` — config.json en `%APPDATA%\Asegurao`, hashes Argon2id, escritura atómica.
- `procctl.rs` — procesos Windows: `Scanner` (procesos de la sesión del usuario, ignora
  servicios de sesión 0, nombre original del .exe cacheado por PID), congelar/reanudar/cerrar
  por PID, PIDs con ventana visible, `protect_self` (DACL que deniega cerrar/congelar el
  proceso), icono real del .exe → PNG.
- `apps.rs` — descubrir apps (registro Uninstall + procesos abiertos).
- `watcher.rs` — hilo cada 250 ms: congela todos los PIDs de apps bloqueadas no permitidas
  (lleva la cuenta en `rt.frozen`, NtSuspendProcess acumula) y pide contraseña de una en una
  (`pending`); el resto espera congelado. Auto-bloqueo de sesión.
- `state.rs` — estado en memoria (Instant). `Allow::WhileShown` (confianza 0: permitida
  mientras se vea; si se esconde 2 s → `dormant`, se bloquea al reaparecer) o `Allow::Until`.
  `Runtime::release` / `release_all` reanudan o cierran lo congelado.
- `history.rs` — historial (máx. 1000, rota) + carpeta `fotos/`.
- `intruder.rs` — captura webcam opcional (nokhwa) tras N fallos.
- `uac.rs` — 2ª llave: `ShellExecuteW runas` (confirmación de Windows).
- `commands.rs` — comandos Tauri. `lib.rs` — plugins, bandeja, ventanas, modo gate.

**Frontend** (`src/`): `+page.svelte` conmuta por etiqueta de ventana
(`main` / `unlock` / `gate`). Componentes en `src/lib/`. Temas por variables CSS
en `app.css` (Grafito por defecto, Bóveda, Aurora). La ventana `unlock` recupera
el aviso al montarse (`unlock_pending`) por si el evento llegó antes de cargar.

**Ventanas:** `main` (asistente/login/panel), `unlock` (tarjeta flotante
transparente, `shadow:false`), `gate` (contraseña al desinstalar).

## Decisiones clave

- **Congela** (suspende) la app en vez de cerrarla; reanuda al acertar.
- **Reforzar** protección = solo sesión. **Debilitar** (quitar app, bajar nivel,
  borrar historial, cambiar maestra, cerrar Asegurao, desinstalar) = maestra **+ UAC**.
- **Nivel Fortaleza** por defecto. PIN mín. 4 dígitos / contraseña mín. 8.
- **Anti-cierre por DACL** (no por elevación): elevar el proceso anularía la 2ª llave
  UAC (un proceso admin pasa `runas` sin diálogo). Solo se cierra desde Ajustes.
- **Sin driver de kernel** ni rootkit. El techo real contra un admin es BitLocker + BIOS.
  Ojo: en cuentas admin el UAC es solo "Sí", así que un admin en tu sesión puede
  abrir el Administrador de tareas elevado y cerrarlo.
- **Desinstalar** pide la maestra vía hook NSIS `PREUNINSTALL` → `--uninstall-gate`.
  Las actualizaciones (modo `/UPDATE`) no pasan por el desinstalador.

## Build y releases

- `deploy.bat` → `deploy-hosting/`: setup + `.sig` + `latest.json` + portable.
- **Firma de actualizaciones**: clave privada en `%USERPROFILE%\.tauri\asegurao.key`
  (sin contraseña), cargada desde `.env` (gitignored). Pública en `tauri.conf.json`.
  Si se pierde la clave, las apps instaladas no aceptarán más actualizaciones.
- Release en GitHub: setup + `.sig` + `latest.json` (el portable no se sube).
  Endpoint: `releases/latest/download/latest.json`.

## Verificado

Tests Rust (5): sesión, exe renombrado, anti-cierre con taskkill. `svelte-check` 0 avisos.

## Pendiente

- Apps que corren como administrador no se pueden congelar (Asegurao no va elevado).
- Si Asegurao se cierra con una app congelada (p. ej. admin), esa app queda congelada.
- Modo Extremo / Fortaleza avanzado (bloqueo de herramientas): descartado por ahora.
- Landing web, capturas en el README, firma de código del .exe.
