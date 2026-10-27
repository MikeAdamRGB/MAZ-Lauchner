<div align="center">

<img src="../../assets/icon-256.png" width="128" alt="Logo di Open Game Launcher">

# Open Game Launcher

**Tutti i tuoi giochi. Un solo launcher.**

Steam, Epic Games e i tuoi giochi in un'unica app per Windows veloce, leggera e curata, scritta al 100% in Rust.

[![Ultima versione](https://img.shields.io/github/v/release/MikeAdamRGB/Open-Game-Launcher?label=Versione&color=8b5cf6)](https://github.com/MikeAdamRGB/Open-Game-Launcher/releases/latest)
[![Download](https://img.shields.io/github/downloads/MikeAdamRGB/Open-Game-Launcher/total?label=Download&color=ec4899)](https://github.com/MikeAdamRGB/Open-Game-Launcher/releases)
[![Build](https://github.com/MikeAdamRGB/Open-Game-Launcher/actions/workflows/build.yml/badge.svg)](https://github.com/MikeAdamRGB/Open-Game-Launcher/actions/workflows/build.yml)
![Windows](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6?logo=windows&logoColor=white)
![Rust](https://img.shields.io/badge/100%25-Rust-DEA584?logo=rust&logoColor=white)
[![Licenza: MIT](https://img.shields.io/badge/Licenza-MIT-22c55e)](../../LICENSE)

[English](../../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · **Italiano** · [Português](README.pt-BR.md) · [Polski](README.pl.md) · [Türkçe](README.tr.md) · [Русский](README.ru.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [简体中文](README.zh-CN.md)

[**⬇️ Scarica per Windows**](https://github.com/MikeAdamRGB/Open-Game-Launcher/releases/latest)

<br>

<img src="../screenshots/library-epic-style.webp" alt="Libreria di Open Game Launcher in stile Epic Games" width="900">

</div>

---

> [!NOTE]
> L'interfaccia dell'app è attualmente in **tedesco**. Le etichette citate qui sotto (ad esempio **«Spiel hinzufügen»**) sono riportate esattamente come appaiono nell'app, con la relativa traduzione.

## Funzionalità

| | |
|---|---|
| 🎮 **Steam** | Rileva automaticamente tutti i giochi Steam installati, su tutti i dischi. Con una chiave Steam Web API opzionale mostra **tutti** i giochi posseduti, compreso il tempo di gioco. |
| 🟦 **Epic Games** | Rileva automaticamente tutti i giochi Epic installati. |
| 🚀 **I tuoi giochi** | Aggiungi qualsiasi gioco: `.exe`, collegamento o file batch. Prima di **ogni** avvio una checklist chiede *«Hai disattivato tutto ciò che è importante?»*. Se rispondi **No**, il gioco non parte. |
| 🖼️ **Icone e copertine originali** | Icone reali (estratte dall'EXE o da Steam) e copertine ufficiali. Puoi impostare una copertina personalizzata per ogni gioco. |
| 🎨 **3 layout** | Si presenta **come la libreria di Steam**, **come Epic Games** oppure nello **stile OG**. |
| 🌈 **8 temi colore** | OG Viola, Mezzanotte, Neon, Brace, Foresta, Sakura, Nero OLED e Chiaro, più un colore d'accento personalizzato. |
| 🛟 **Errori spiegati in modo semplice** | Se qualcosa non parte, il launcher spiega **passo dopo passo** cosa è successo e come risolverlo, con pulsanti che aiutano subito (ad esempio *Esegui come amministratore* o *Scarica Visual C++*). |
| 📦 **Installer integrato** | Un unico EXE: con un doppio clic si apre un menu di installazione con collegamento sul desktop, voce nel menu Start, *Apri ora* e *Apri cartella dell'app*. Non servono diritti di amministratore. |
| 🪶 **Leggerissimo** | Un solo EXE ottimizzato per le dimensioni, senza file aggiuntivi. |

## Screenshot

<table>
  <tr>
    <td width="50%" align="center">
      <img src="../screenshots/library-steam-style.webp" alt="Libreria in stile Steam"><br>
      <sub><b>Stile Steam</b>: elenco giochi a sinistra, «Giocati di recente» e griglia di tutti i giochi</sub>
    </td>
    <td width="50%" align="center">
      <img src="../screenshots/library-epic-style.webp" alt="Libreria in stile Epic Games"><br>
      <sub><b>Stile Epic Games</b>: schede grandi e filtri in alto</sub>
    </td>
  </tr>
  <tr>
    <td width="50%" align="center">
      <img src="../screenshots/game-details.webp" alt="Dettagli del gioco"><br>
      <sub><b>Dettagli del gioco</b>: gioca, preferiti, apri cartella, copertina personalizzata</sub>
    </td>
    <td width="50%" align="center">
      <img src="../screenshots/settings-design.webp" alt="Impostazioni di design"><br>
      <sub><b>Impostazioni di design</b>: 3 layout, 8 temi e colore d'accento</sub>
    </td>
  </tr>
</table>

## Installazione

**Requisiti:** Windows 10 o 11 (64 bit) con [Microsoft WebView2](https://developer.microsoft.com/microsoft-edge/webview2/), preinstallato nelle versioni attuali di Windows.

1. Scarica l'ultimo **`Open-Game-Launcher.exe`** da [**Releases**](https://github.com/MikeAdamRGB/Open-Game-Launcher/releases/latest).
2. Fai **doppio clic** sul file.
3. Clicca su **«Installieren»** (*Installa*) nel menu di installazione.
4. Fatto! Clicca su **«Jetzt öffnen»** (*Apri ora*). Da quel momento avvia il launcher dal collegamento sul desktop.

> [!TIP]
> **Windows mostra «PC protetto da Windows»?**
> Succede con i programmi nuovi non firmati con un certificato a pagamento. Clicca su **Ulteriori informazioni** e poi su **Esegui comunque**.

> [!TIP]
> **Preferisci la versione portable?** Clicca su **«Ohne Installation starten»** (*Avvia senza installare*) nel menu di installazione, oppure metti un file vuoto chiamato `portable.txt` accanto all'EXE.


**Disinstallazione:** Impostazioni di Windows → App → «Open Game Launcher» → Disinstalla, oppure nel launcher in **Einstellungen → Allgemein** (*Impostazioni → Generale*). I tuoi giochi restano intatti.

## Utilizzo

- **Giocare:** passa il mouse su un gioco e clicca su ▶. Un clic sulla copertina apre i dettagli.
- **Aggiungere un gioco:** clicca su **«Spiel hinzufügen»** (*Aggiungi gioco*) a sinistra → **«Durchsuchen…»** (*Sfoglia…*) accanto a «Programm» → scegli l'eseguibile del gioco → **«Hinzufügen»** (*Aggiungi*).
- **Cambiare aspetto:** **Einstellungen → Design** → scegli un layout (OG / Steam / Epic) e un tema colore.
- **Vedere tutti i giochi Steam, anche non installati:** **Einstellungen → Verbindungen** (*Connessioni*) → inserisci una chiave Steam Web API gratuita. Le istruzioni sono proprio lì accanto.
- **Personalizzare la checklist:** **Einstellungen → Eigene Spiele** (*Giochi personali*) → aggiungi, modifica o elimina voci, in generale o per singolo gioco.

### Errori spiegati automaticamente

File del gioco mancante · unità o disco USB non collegato · servono i diritti di amministratore · accesso negato · file sbagliato · bloccato dall'antivirus · bloccato da Windows / Smart App Control · file in uso · DLL mancanti (Visual C++ / DirectX) · errore **0xc000007b** · **0xc0000142** · crash subito dopo l'avvio · Steam o Epic non installati · Steam o Epic non rispondono · chiave Steam API non valida · WebView2 mancante · installazione non riuscita.

## Compilare dai sorgenti

**Requisiti:** [Rust](https://rustup.rs) (stable, 1.85 o successivo) su Windows. Nient'altro: niente Node.js, niente CLI di Dioxus.

```powershell
git clone https://github.com/MikeAdamRGB/Open-Game-Launcher.git
cd MAZ-Lauchner
cargo run                 # sviluppo
cargo build --release     # EXE finale: target\release\Open-Game-Launcher.exe
.\scripts\build.ps1 -Mini # copia l'EXE in dist\ e crea una versione mini compressa con UPX
```

Il foglio di stile compilato (`assets/maz.css`) è già nel repository. Se modifichi le classi Tailwind, esegui `.\scripts\build-css.ps1`.

Dettagli tecnici (stack, struttura del progetto, ottimizzazione delle dimensioni): vedi il [README in inglese](../../README.md#tech-stack).

## FAQ

**Il launcher scarica giochi o aggira le protezioni anticopia?**
No. I giochi Steam ed Epic vengono sempre avviati tramite i launcher ufficiali. Non vengono mai richieste password.

**Perché non vedo tutti i miei giochi Epic?**
Epic fornisce l'elenco dei giochi acquistati solo dopo l'accesso. Open Game Launcher mostra tutti i giochi Epic **installati**.

**La finestra resta bianca o non si apre.**
Il launcher richiede Microsoft WebView2 (normalmente incluso in Windows 10 e 11). Se manca, all'avvio il launcher spiega come installarlo.

**I miei dati vengono inviati da qualche parte?**
Nessun account e nessuna telemetria. La rete viene usata solo per le copertine e, se aggiungi una chiave, per la Steam Web API.

## Contribuire

Segnalazioni di bug, idee e pull request sono benvenute: [apri una issue](https://github.com/MikeAdamRGB/Open-Game-Launcher/issues/new/choose) · [CONTRIBUTING.md](../../CONTRIBUTING.md)


## Estensioni e Minecraft

**Minecraft e NoRisk Client:** il Minecraft Launcher ufficiale e ogni profilo NoRisk compaiono come riquadri. Dalla scheda «Minecraft» installi mod, texture pack, shader e data pack da Modrinth e CurseForge.

**Estensioni** (disattivate di default): piccoli programmi esterni che aggiungono fonti di giochi, info o pulsanti. Prima dell'installazione vedi tutti i permessi e il checksum. Nota onesta: girano con i tuoi permessi utente – installa solo quelle di cui ti fidi. Dettagli: [docs/EXTENSIONS.md](../EXTENSIONS.md).

## Licenza

Rilasciato con [licenza MIT](../../LICENSE) © MikeAdamRGB.

Design dei componenti ispirato a [rust-ui](https://github.com/rust-ui/ui) (MIT).
Steam è un marchio di Valve Corporation. Epic Games è un marchio di Epic Games, Inc. Questo progetto è indipendente e non è affiliato a nessuna delle due aziende.
