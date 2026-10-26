<div align="center">

<img src="../../assets/icon-256.png" width="128" alt="Logo de MAZ Launcher">

# MAZ Launcher

**Todos tus juegos. Un solo launcher.**

Steam, Epic Games y tus propios juegos en una app para Windows rápida, ligera y bonita, escrita al 100 % en Rust.

[![Última versión](https://img.shields.io/github/v/release/MikeAdamRGB/MAZ-Lauchner?label=Versi%C3%B3n&color=8b5cf6)](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases/latest)
[![Descargas](https://img.shields.io/github/downloads/MikeAdamRGB/MAZ-Lauchner/total?label=Descargas&color=ec4899)](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases)
[![Build](https://github.com/MikeAdamRGB/MAZ-Lauchner/actions/workflows/build.yml/badge.svg)](https://github.com/MikeAdamRGB/MAZ-Lauchner/actions/workflows/build.yml)
![Windows](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6?logo=windows&logoColor=white)
![Rust](https://img.shields.io/badge/100%25-Rust-DEA584?logo=rust&logoColor=white)
[![Licencia: MIT](https://img.shields.io/badge/Licencia-MIT-22c55e)](../../LICENSE)

[English](../../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · **Español** · [Italiano](README.it.md) · [Português](README.pt-BR.md) · [Polski](README.pl.md) · [Türkçe](README.tr.md) · [Русский](README.ru.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [简体中文](README.zh-CN.md)

[**⬇️ Descargar para Windows**](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases/latest)

<br>

<img src="../screenshots/library-epic-style.webp" alt="Biblioteca de MAZ Launcher con el estilo de Epic Games" width="900">

</div>

---

> [!NOTE]
> La interfaz de la aplicación está actualmente en **alemán**. Las etiquetas citadas abajo (por ejemplo **«Spiel hinzufügen»**) aparecen tal como se ven en la app, junto con su traducción.

## Funciones

| | |
|---|---|
| 🎮 **Steam** | Detecta automáticamente todos los juegos de Steam instalados, en todas las unidades. Con una clave opcional de la Steam Web API muestra **todos** los juegos que posees, incluido el tiempo de juego. |
| 🟦 **Epic Games** | Detecta automáticamente todos los juegos de Epic instalados. |
| 🚀 **Tus propios juegos** | Añade cualquier juego: `.exe`, acceso directo o archivo batch. Antes de **cada** inicio, una lista de comprobación pregunta *«¿Has desactivado todo lo importante?»*. Si respondes **No**, el juego no se inicia. |
| 🖼️ **Iconos y portadas originales** | Iconos reales (extraídos del EXE o de Steam) y portadas oficiales. Puedes poner una portada personalizada a cualquier juego. |
| 🎨 **3 diseños de biblioteca** | Se ve **como la biblioteca de Steam**, **como Epic Games** o con el **estilo MAZ** propio. |
| 🌈 **8 temas de color** | MAZ Violeta, Medianoche, Neón, Brasa, Bosque, Sakura, Negro OLED y Claro, además de un color de acento personalizado. |
| 🛟 **Errores explicados con claridad** | Si algo no arranca, el launcher explica **paso a paso** qué ha pasado y cómo solucionarlo, con botones que ayudan directamente (por ejemplo *Ejecutar como administrador* o *Descargar Visual C++*). |
| 📦 **Instalador integrado** | Un único EXE: con doble clic se abre un menú de instalación con acceso directo en el escritorio, entrada en el menú Inicio, *Abrir ahora* y *Abrir carpeta de la app*. No requiere permisos de administrador. |
| 🪶 **Muy ligero** | Un solo EXE optimizado en tamaño, sin archivos adicionales. |

## Capturas de pantalla

<table>
  <tr>
    <td width="50%" align="center">
      <img src="../screenshots/library-steam-style.webp" alt="Biblioteca con el estilo de Steam"><br>
      <sub><b>Estilo Steam</b>: lista de juegos a la izquierda, «Jugado recientemente» y cuadrícula de juegos</sub>
    </td>
    <td width="50%" align="center">
      <img src="../screenshots/library-epic-style.webp" alt="Biblioteca con el estilo de Epic Games"><br>
      <sub><b>Estilo Epic Games</b>: tarjetas grandes y filtros arriba</sub>
    </td>
  </tr>
  <tr>
    <td width="50%" align="center">
      <img src="../screenshots/game-details.webp" alt="Detalles del juego"><br>
      <sub><b>Detalles del juego</b>: jugar, favoritos, abrir carpeta, portada personalizada</sub>
    </td>
    <td width="50%" align="center">
      <img src="../screenshots/settings-design.webp" alt="Ajustes de diseño"><br>
      <sub><b>Ajustes de diseño</b>: 3 diseños, 8 temas y color de acento</sub>
    </td>
  </tr>
</table>

## Instalación

**Requisitos:** Windows 10 u 11 (64 bits) con [Microsoft WebView2](https://developer.microsoft.com/microsoft-edge/webview2/), que viene preinstalado en las versiones actuales de Windows.

1. Descarga el último **`MAZ-Launcher.exe`** desde [**Releases**](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases/latest).
2. Haz **doble clic** en el archivo.
3. Pulsa **«Installieren»** (*Instalar*) en el menú de instalación.
4. ¡Listo! Pulsa **«Jetzt öffnen»** (*Abrir ahora*). A partir de entonces, abre el launcher desde el acceso directo del escritorio.

> [!TIP]
> **¿Windows muestra «Windows protegió su PC»?**
> Ocurre con programas nuevos que no están firmados con un certificado de pago. Pulsa **Más información** y luego **Ejecutar de todas formas**.

> [!TIP]
> **¿Prefieres una versión portable?** Pulsa **«Ohne Installation starten»** (*Iniciar sin instalar*) en el menú de instalación, o coloca un archivo vacío llamado `portable.txt` junto al EXE.

**`MAZ-Launcher-mini.exe`** es la misma app comprimida con UPX. Algunos antivirus marcan por error los EXE comprimidos; en ese caso, usa la versión normal.

**Desinstalar:** Configuración de Windows → Aplicaciones → «MAZ Launcher» → Desinstalar, o en el launcher en **Einstellungen → Allgemein** (*Ajustes → General*). Tus juegos no se tocan.

## Uso

- **Jugar:** pasa el ratón por encima de un juego y pulsa ▶. Un clic en la portada abre los detalles.
- **Añadir un juego propio:** pulsa **«Spiel hinzufügen»** (*Añadir juego*) a la izquierda → **«Durchsuchen…»** (*Examinar…*) junto a «Programm» → elige el ejecutable del juego → **«Hinzufügen»** (*Añadir*).
- **Cambiar el aspecto:** **Einstellungen → Design** → elige un diseño (MAZ / Steam / Epic) y un tema de color.
- **Ver todos los juegos de Steam, incluso los no instalados:** **Einstellungen → Verbindungen** (*Conexiones*) → introduce una clave gratuita de la Steam Web API. Las instrucciones aparecen justo al lado.
- **Personalizar la lista de comprobación:** **Einstellungen → Eigene Spiele** (*Juegos propios*) → añade, edita o elimina puntos, de forma general o por juego.

### Errores que el launcher explica automáticamente

Falta el archivo del juego · unidad o disco USB no conectado · se necesitan permisos de administrador · acceso denegado · archivo incorrecto · bloqueado por el antivirus · bloqueado por Windows / Smart App Control · archivo en uso · faltan DLL (Visual C++ / DirectX) · error **0xc000007b** · **0xc0000142** · cierre justo después de iniciar · Steam o Epic no instalado · Steam o Epic no responde · clave de Steam API no válida · falta WebView2 · la instalación ha fallado.

## Compilar desde el código fuente

**Requisitos:** [Rust](https://rustup.rs) (stable, 1.85 o superior) en Windows. Nada más: ni Node.js ni la CLI de Dioxus.

```powershell
git clone https://github.com/MikeAdamRGB/MAZ-Lauchner.git
cd MAZ-Lauchner
cargo run                 # desarrollo
cargo build --release     # EXE final: target\release\MAZ-Launcher.exe
.\scripts\build.ps1 -Mini # copia el EXE a dist\ y crea una versión mini comprimida con UPX
```

La hoja de estilos compilada (`assets/maz.css`) ya está en el repositorio. Si cambias clases de Tailwind, ejecuta `.\scripts\build-css.ps1`.

Detalles técnicos (stack, estructura del proyecto, optimización de tamaño): consulta el [README en inglés](../../README.md#tech-stack).

## Preguntas frecuentes

**¿El launcher descarga juegos o se salta la protección anticopia?**
No. Los juegos de Steam y Epic siempre se inician con sus launchers oficiales. Nunca se piden contraseñas.

**¿Por qué no veo todos mis juegos de Epic?**
Epic solo entrega la lista de juegos comprados tras iniciar sesión. MAZ Launcher muestra todos los juegos de Epic **instalados**.

**La ventana se queda en blanco o no se abre.**
El launcher necesita Microsoft WebView2 (normalmente incluido en Windows 10 y 11). Si falta, el launcher explica al iniciarse cómo instalarlo.

**¿Se envían mis datos a algún sitio?**
Sin cuentas ni telemetría. La red solo se usa para las portadas y, si añades una clave, para la Steam Web API.

## Contribuir

Se agradecen informes de errores, ideas y pull requests: [abrir un issue](https://github.com/MikeAdamRGB/MAZ-Lauchner/issues/new/choose) · [CONTRIBUTING.md](../../CONTRIBUTING.md)


## Extensiones y Minecraft

**Minecraft y NoRisk Client:** el Minecraft Launcher oficial y cada perfil de NoRisk aparecen como mosaicos propios. En la pestaña «Minecraft» instalas mods, paquetes de texturas, shaders y paquetes de datos de Modrinth y CurseForge.

**Extensiones** (desactivadas por defecto): pequeños programas externos que añaden fuentes de juegos, información o botones. Antes de instalar ves todos los permisos y la suma de verificación. Aviso sincero: se ejecutan con tus permisos de usuario; instala solo las de confianza. Detalles: [docs/EXTENSIONS.md](../EXTENSIONS.md).

## Licencia

Publicado bajo la [licencia MIT](../../LICENSE) © MikeAdamRGB.

Diseño de componentes inspirado en [rust-ui](https://github.com/rust-ui/ui) (MIT).
Steam es una marca de Valve Corporation. Epic Games es una marca de Epic Games, Inc. Este proyecto es independiente y no está afiliado a ninguna de ellas.
