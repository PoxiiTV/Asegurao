# 🛡️ Asegurao

Protege tus aplicaciones de Windows con contraseña. Cuando alguien intenta abrir
una app protegida, Asegurao la **congela** al instante y pide la clave para
continuar.

Reescritura completa y en español de [AppLocker](https://github.com/Hachiiki/AppLocker),
con interfaz nueva (Tauri + Svelte), temas, historial y más seguridad.

*(English below · [English version](#-asegurao-english))*

---

## ✨ Características

- **Contraseña maestra** para entrar y autorizar cambios (hash **Argon2id**).
- **Contraseña por app**, propia o reutilizando la maestra.
- **Congela** la app en vez de cerrarla: al acertar, sigue donde estaba.
- **Tiempo de confianza**: tras desbloquear, no volver a pedir la clave durante
  5 / 15 / 60 min.
- **Historial** de intentos (concedidos, denegados, manipulación).
- **Foto del intruso** opcional (webcam) tras varios fallos.
- **Añade apps** desde las instaladas, las abiertas ahora o cualquier `.exe`.
- **3 temas**: Grafito, Bóveda y Aurora.
- Vive en la **bandeja del sistema** y arranca con Windows.
- Nivel **Fortaleza**: para desactivar la protección, cambiar la clave o
  desinstalar, Windows pide **también** tu contraseña (UAC).

## 🔒 Hasta dónde protege (sé realista)

Asegurao frena de sobra a cualquiera que use tu ordenador de forma normal:
hermanos, hijos, compañeros de piso, un curioso de paso.

Lo que **ningún** programa puede impedir por sí solo es que una persona que:

- **sea administrador**, **sepa tu contraseña de Windows** y tenga conocimientos
  avanzados,

arranque desde un USB o saque el disco. Contra ese caso extremo la única defensa
real es **BitLocker + contraseña de BIOS** (configuración de Windows, no de esta
app). Cualquiera que prometa "imposible de saltar" sin eso, miente.

## 🖥️ Requisitos

- Windows 10 / 11
- Permisos de administrador (para congelar y relanzar procesos)

## 🚀 Desarrollo

```bash
npm install
npm run tauri dev      # o doble clic en start.bat
```

## 📦 Compilar

```bash
npm run tauri build    # o doble clic en deploy.bat
```

El instalador queda en `src-tauri/target/release/bundle/nsis/`
(`deploy.bat` además lo copia a `deploy-hosting/`).

> ⚠️ Al ser un `.exe` sin firmar que controla otros procesos, Windows Defender o
> SmartScreen pueden avisar la primera vez ("Windows protegió su PC" → *Más
> información* → *Ejecutar de todas formas*). Es normal en software de este tipo.

## 🛠️ Tecnología

Tauri 2 · Rust · Svelte 5 · Vite

---

# 🛡️ Asegurao (English)

Password-protect your Windows apps. When someone tries to open a protected app,
Asegurao **freezes** it instantly and asks for the password to continue.

A full Spanish rewrite of [AppLocker](https://github.com/Hachiiki/AppLocker) with
a brand-new UI (Tauri + Svelte), themes, history and stronger security.

## ✨ Features

- **Master password** to enter and authorize changes (**Argon2id** hashing).
- **Per-app password**, custom or reusing the master one.
- **Freezes** the app instead of killing it: unlock and it resumes where it was.
- **Trust window**: after unlocking, don't ask again for 5 / 15 / 60 min.
- **Attempt history** (allowed, denied, tampering).
- Optional **intruder photo** (webcam) after several failures.
- **Add apps** from installed ones, currently running ones, or any `.exe`.
- **3 themes**: Grafito, Bóveda, Aurora.
- Lives in the **system tray** and starts with Windows.
- **Fortress** level: disabling protection, changing the password or
  uninstalling also requires your Windows password (UAC).

## 🔒 How far it protects (be realistic)

Asegurao easily stops anyone using your computer normally. What **no** program
alone can stop is an **administrator who knows your Windows password** and has
advanced skills booting from USB or removing the drive — only **BitLocker + BIOS
password** defends against that (a Windows setting, not this app).

## 🖥️ Requirements

- Windows 10 / 11
- Administrator rights (to freeze and relaunch processes)

## 🚀 Development / Build

```bash
npm install
npm run tauri dev      # run
npm run tauri build    # build installer
```

## 🛠️ Stack

Tauri 2 · Rust · Svelte 5 · Vite

---

Basado en AppLocker de Hachiki · Licencia MIT
