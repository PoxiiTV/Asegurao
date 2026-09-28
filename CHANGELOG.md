# Changelog

## 🇪🇸 Español

### 1.1.0

**Seguridad**
- **No se puede cerrar** desde el Administrador de tareas ni con `taskkill` (*Acceso denegado*). Nuevo botón **Cerrar Asegurao** en Ajustes, que pide la contraseña maestra.
- **Detecta copias renombradas** de una app bloqueada (lee el nombre original del `.exe`).
- Si hay una contraseña pendiente, **las demás apps bloqueadas también se congelan** y esperan su turno (antes se podían abrir en ese momento).
- **Apps de la bandeja** (Telegram, Discord…): al esconderlas, vuelven a pedir la clave cuando reaparecen.

**Mejoras**
- **Actualizaciones automáticas** firmadas desde Ajustes → *Buscar actualizaciones*.
- Vigilancia más ligera: una sola lectura de procesos por vuelta.
- Ignora los servicios de Windows (p. ej. el de Everything), que bloqueaban la vigilancia.

### 1.0.0

- Primera versión: bloqueo de apps con contraseña, contraseña maestra y por app, tiempo de confianza, historial, foto del intruso, 3 temas, nivel Fortaleza y desinstalación protegida.

---

## 🇬🇧 English

### 1.1.0

**Security**
- **Can't be killed** from Task Manager or `taskkill` (*Access denied*). New **Close Asegurao** button in Settings, protected by the master password.
- **Catches renamed copies** of a locked app (reads the `.exe` original filename).
- While a password prompt is open, **other locked apps are frozen too** and wait their turn (they used to slip through).
- **Tray apps** (Telegram, Discord…): once hidden, they ask for the password again when they reappear.

**Improvements**
- **Signed auto-updates** from Settings → *Check for updates*.
- Lighter watcher: a single process snapshot per cycle.
- Ignores Windows services (e.g. Everything's), which used to stall the watcher.

### 1.0.0

- First release: password-locked apps, master and per-app passwords, trust window, history, intruder photo, 3 themes, Fortress level and protected uninstall.
