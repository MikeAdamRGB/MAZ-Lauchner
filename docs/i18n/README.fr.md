<div align="center">

<img src="../../assets/icon-256.png" width="128" alt="Logo Open Game Launcher">

# Open Game Launcher

**Tous vos jeux. Un seul launcher.**

Steam, Epic Games et vos propres jeux réunis dans une application Windows rapide, légère et élégante, écrite à 100 % en Rust.

[![Dernière version](https://img.shields.io/github/v/release/MikeAdamRGB/Open-Game-Launcher?label=Version&color=8b5cf6)](https://github.com/MikeAdamRGB/Open-Game-Launcher/releases/latest)
[![Téléchargements](https://img.shields.io/github/downloads/MikeAdamRGB/Open-Game-Launcher/total?label=T%C3%A9l%C3%A9chargements&color=ec4899)](https://github.com/MikeAdamRGB/Open-Game-Launcher/releases)
[![Build](https://github.com/MikeAdamRGB/Open-Game-Launcher/actions/workflows/build.yml/badge.svg)](https://github.com/MikeAdamRGB/Open-Game-Launcher/actions/workflows/build.yml)
![Windows](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6?logo=windows&logoColor=white)
![Rust](https://img.shields.io/badge/100%25-Rust-DEA584?logo=rust&logoColor=white)
[![Licence : MIT](https://img.shields.io/badge/Licence-MIT-22c55e)](../../LICENSE)

[English](../../README.md) · [Deutsch](README.de.md) · **Français** · [Español](README.es.md) · [Italiano](README.it.md) · [Português](README.pt-BR.md) · [Polski](README.pl.md) · [Türkçe](README.tr.md) · [Русский](README.ru.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [简体中文](README.zh-CN.md)

[**⬇️ Télécharger pour Windows**](https://github.com/MikeAdamRGB/Open-Game-Launcher/releases/latest)

<br>

<img src="../screenshots/library-epic-style.webp" alt="Bibliothèque Open Game Launcher dans le style Epic Games" width="900">

</div>

---

> [!NOTE]
> L'interface de l'application est actuellement en **allemand**. Les libellés cités ci-dessous (par exemple **« Spiel hinzufügen »**) sont reproduits tels qu'ils apparaissent dans l'application, avec leur traduction.

## Fonctionnalités

| | |
|---|---|
| 🎮 **Steam** | Détecte automatiquement tous les jeux Steam installés, sur tous les disques. Avec une clé API Steam Web (facultative), affiche **tous** les jeux possédés, temps de jeu compris. |
| 🟦 **Epic Games** | Détecte automatiquement tous les jeux Epic installés. |
| 🚀 **Vos propres jeux** | Ajoutez n'importe quel jeu : `.exe`, raccourci ou fichier batch. Avant **chaque** lancement, une liste de vérification demande *« As-tu désactivé tout ce qui est important ? »*. Si vous répondez **Non**, le jeu ne démarre pas. |
| 🖼️ **Icônes et jaquettes d'origine** | Vraies icônes (extraites de l'EXE ou de Steam) et jaquettes officielles. Vous pouvez définir une jaquette personnalisée pour chaque jeu. |
| 🎨 **3 dispositions** | Ressemble à **la bibliothèque Steam**, à **Epic Games**, ou adopte le **style OG** propre au launcher. |
| 🌈 **8 thèmes de couleurs** | OG Violet, Minuit, Néon, Braise, Forêt, Sakura, Noir OLED et Clair, plus une couleur d'accent personnalisée. |
| 🛟 **Erreurs expliquées simplement** | Si quelque chose ne démarre pas, le launcher explique **étape par étape** ce qui s'est passé et comment corriger le problème, avec des boutons qui agissent directement (par exemple *Exécuter en tant qu'administrateur* ou *Télécharger Visual C++*). |
| 📦 **Installateur intégré** | Un seul EXE : un double-clic ouvre un menu d'installation avec raccourci bureau, entrée dans le menu Démarrer, *Ouvrir maintenant* et *Ouvrir le dossier de l'application*. Aucun droit administrateur requis. |
| 🪶 **Ultra léger** | Un seul EXE optimisé pour la taille, sans fichiers supplémentaires. |

## Captures d'écran

<table>
  <tr>
    <td width="50%" align="center">
      <img src="../screenshots/library-steam-style.webp" alt="Bibliothèque dans le style Steam"><br>
      <sub><b>Style Steam</b> : liste des jeux à gauche, « Joués récemment » et grille de tous les jeux</sub>
    </td>
    <td width="50%" align="center">
      <img src="../screenshots/library-epic-style.webp" alt="Bibliothèque dans le style Epic Games"><br>
      <sub><b>Style Epic Games</b> : grandes cartes et filtres en haut</sub>
    </td>
  </tr>
  <tr>
    <td width="50%" align="center">
      <img src="../screenshots/game-details.webp" alt="Détails d'un jeu"><br>
      <sub><b>Détails du jeu</b> : jouer, ajouter aux favoris, ouvrir le dossier, choisir une jaquette</sub>
    </td>
    <td width="50%" align="center">
      <img src="../screenshots/settings-design.webp" alt="Paramètres d'apparence"><br>
      <sub><b>Paramètres d'apparence</b> : 3 dispositions, 8 thèmes et couleur d'accent</sub>
    </td>
  </tr>
</table>

## Installation

**Prérequis :** Windows 10 ou 11 (64 bits) avec [Microsoft WebView2](https://developer.microsoft.com/microsoft-edge/webview2/), préinstallé sur les versions récentes de Windows.

1. Téléchargez le dernier **`Open-Game-Launcher.exe`** depuis les [**Releases**](https://github.com/MikeAdamRGB/Open-Game-Launcher/releases/latest).
2. **Double-cliquez** sur le fichier.
3. Cliquez sur **« Installieren »** (*Installer*) dans le menu d'installation.
4. C'est fait ! Cliquez sur **« Jetzt öffnen »** (*Ouvrir maintenant*). Ensuite, lancez le launcher depuis le raccourci du bureau.

> [!TIP]
> **Windows affiche « Windows a protégé votre ordinateur » ?**
> Cela arrive avec les nouveaux programmes qui ne sont pas signés par un certificat payant. Cliquez sur **Informations complémentaires**, puis sur **Exécuter quand même**.

> [!TIP]
> **Version portable ?** Cliquez sur **« Ohne Installation starten »** (*Démarrer sans installer*) dans le menu d'installation, ou placez un fichier vide nommé `portable.txt` à côté de l'EXE.


**Désinstallation :** Paramètres Windows → Applications → « Open Game Launcher » → Désinstaller, ou dans le launcher sous **Einstellungen → Allgemein** (*Paramètres → Général*). Vos jeux ne sont pas touchés.

## Utilisation

- **Jouer :** survolez un jeu et cliquez sur ▶. Un clic sur la jaquette ouvre les détails.
- **Ajouter un jeu :** cliquez sur **« Spiel hinzufügen »** (*Ajouter un jeu*) à gauche → **« Durchsuchen… »** (*Parcourir…*) à côté de « Programm » → choisissez l'exécutable du jeu → **« Hinzufügen »** (*Ajouter*).
- **Changer l'apparence :** **Einstellungen → Design** → choisissez une disposition (OG / Steam / Epic) et un thème.
- **Voir tous les jeux Steam, même non installés :** **Einstellungen → Verbindungen** (*Connexions*) → saisissez une clé API Steam Web gratuite. Les instructions sont affichées juste à côté.
- **Personnaliser la liste de vérification :** **Einstellungen → Eigene Spiele** (*Mes jeux*) → ajoutez, modifiez ou supprimez des éléments, globalement ou par jeu.

### Erreurs expliquées automatiquement

Fichier du jeu manquant · disque ou clé USB non branché · droits administrateur requis · accès refusé · mauvais fichier choisi · bloqué par l'antivirus · bloqué par Windows / Smart App Control · fichier en cours d'utilisation · DLL manquantes (Visual C++ / DirectX) · erreur **0xc000007b** · **0xc0000142** · plantage juste après le lancement · Steam ou Epic non installé · Steam ou Epic ne répond pas · clé API Steam invalide · WebView2 manquant · échec de l'installation.

## Compiler soi-même

**Prérequis :** [Rust](https://rustup.rs) (stable, 1.85 ou plus récent) sous Windows. Rien d'autre : ni Node.js, ni CLI Dioxus.

```powershell
git clone https://github.com/MikeAdamRGB/Open-Game-Launcher.git
cd MAZ-Lauchner
cargo run                 # développement
cargo build --release     # EXE final : target\release\Open-Game-Launcher.exe
.\scripts\build.ps1 -Mini # copie l'EXE dans dist\ et crée une version mini compressée avec UPX
```

La feuille de style compilée (`assets/maz.css`) est déjà dans le dépôt. Après une modification des classes Tailwind, lancez `.\scripts\build-css.ps1`.

Détails techniques (stack, structure du projet, optimisation de la taille) : voir le [README anglais](../../README.md#tech-stack).

## FAQ

**Le launcher télécharge-t-il des jeux ou contourne-t-il une protection anticopie ?**
Non. Les jeux Steam et Epic sont toujours lancés via leurs launchers officiels. Aucun mot de passe n'est demandé.

**Pourquoi je ne vois pas tous mes jeux Epic ?**
Epic ne fournit la liste des jeux possédés qu'après connexion. Open Game Launcher affiche tous les jeux Epic **installés**.

**La fenêtre reste blanche ou ne s'ouvre pas.**
Le launcher a besoin de Microsoft WebView2 (normalement inclus dans Windows 10 et 11). S'il manque, le launcher explique au démarrage comment l'installer.

**Mes données sont-elles envoyées quelque part ?**
Pas de compte, pas de télémétrie. Le réseau sert uniquement aux jaquettes et, si vous ajoutez une clé, à l'API Steam Web.

## Contribuer

Rapports de bugs, idées et pull requests sont les bienvenus : [ouvrir une issue](https://github.com/MikeAdamRGB/Open-Game-Launcher/issues/new/choose) · [CONTRIBUTING.md](../../CONTRIBUTING.md)


## Extensions et Minecraft

**Minecraft et NoRisk Client :** le Minecraft Launcher officiel et chaque profil NoRisk apparaissent comme tuiles. L'onglet « Minecraft » installe mods, packs de textures, shaders et packs de données depuis Modrinth et CurseForge.

**Extensions** (désactivées par défaut) : petits programmes externes qui ajoutent des sources de jeux, des infos ou des boutons. Avant l'installation, tu vois tous les droits et la somme de contrôle. Note honnête : elles tournent avec tes droits d'utilisateur – n'installe que des extensions de confiance. Détails : [docs/EXTENSIONS.md](../EXTENSIONS.md).

## Licence

Distribué sous [licence MIT](../../LICENSE) © MikeAdamRGB.

Design des composants inspiré de [rust-ui](https://github.com/rust-ui/ui) (MIT).
Steam est une marque de Valve Corporation. Epic Games est une marque d'Epic Games, Inc. Ce projet est indépendant et n'est affilié à aucune de ces sociétés.
