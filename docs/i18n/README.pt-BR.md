<div align="center">

<img src="../../assets/icon-256.png" width="128" alt="Logo do MAZ Launcher">

# MAZ Launcher

**Todos os seus jogos. Um só launcher.**

Steam, Epic Games e seus próprios jogos em um app para Windows rápido, leve e bonito, escrito 100% em Rust.

[![Última versão](https://img.shields.io/github/v/release/MikeAdamRGB/MAZ-Lauchner?label=Vers%C3%A3o&color=8b5cf6)](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/MikeAdamRGB/MAZ-Lauchner/total?label=Downloads&color=ec4899)](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases)
[![Build](https://github.com/MikeAdamRGB/MAZ-Lauchner/actions/workflows/build.yml/badge.svg)](https://github.com/MikeAdamRGB/MAZ-Lauchner/actions/workflows/build.yml)
![Windows](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6?logo=windows&logoColor=white)
![Rust](https://img.shields.io/badge/100%25-Rust-DEA584?logo=rust&logoColor=white)
[![Licença: MIT](https://img.shields.io/badge/Licen%C3%A7a-MIT-22c55e)](../../LICENSE)

[English](../../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Italiano](README.it.md) · **Português** · [Polski](README.pl.md) · [Türkçe](README.tr.md) · [Русский](README.ru.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [简体中文](README.zh-CN.md)

[**⬇️ Baixar para Windows**](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases/latest)

<br>

<img src="../screenshots/library-epic-style.webp" alt="Biblioteca do MAZ Launcher no estilo Epic Games" width="900">

</div>

---

> [!NOTE]
> A interface do app está atualmente em **alemão**. Os rótulos citados abaixo (por exemplo **“Spiel hinzufügen”**) aparecem exatamente como no app, acompanhados da tradução.

## Recursos

| | |
|---|---|
| 🎮 **Steam** | Detecta automaticamente todos os jogos Steam instalados, em todos os discos. Com uma chave opcional da Steam Web API, mostra **todos** os jogos que você possui, incluindo o tempo de jogo. |
| 🟦 **Epic Games** | Detecta automaticamente todos os jogos Epic instalados. |
| 🚀 **Seus próprios jogos** | Adicione qualquer jogo: `.exe`, atalho ou arquivo batch. Antes de **cada** início, uma checklist pergunta *“Você desativou tudo o que é importante?”*. Se a resposta for **Não**, o jogo não inicia. |
| 🖼️ **Ícones e capas originais** | Ícones reais (extraídos do EXE ou da Steam) e capas oficiais. Você pode definir uma capa personalizada para qualquer jogo. |
| 🎨 **3 layouts** | Fica **como a biblioteca da Steam**, **como a Epic Games** ou no **estilo MAZ** próprio. |
| 🌈 **8 temas de cores** | MAZ Violeta, Meia-noite, Neon, Brasa, Floresta, Sakura, Preto OLED e Claro, além de uma cor de destaque personalizada. |
| 🛟 **Erros explicados de forma simples** | Se algo não abrir, o launcher explica **passo a passo** o que aconteceu e como resolver, com botões que ajudam na hora (por exemplo *Executar como administrador* ou *Baixar Visual C++*). |
| 📦 **Instalador embutido** | Um único EXE: um clique duplo abre o menu de instalação com atalho na área de trabalho, entrada no menu Iniciar, *Abrir agora* e *Abrir pasta do app*. Não precisa de permissão de administrador. |
| 🪶 **Levíssimo** | Um único EXE otimizado em tamanho, sem arquivos extras. |

## Capturas de tela

<table>
  <tr>
    <td width="50%" align="center">
      <img src="../screenshots/library-steam-style.webp" alt="Biblioteca no estilo Steam"><br>
      <sub><b>Estilo Steam</b>: lista de jogos à esquerda, “Jogados recentemente” e grade com todos os jogos</sub>
    </td>
    <td width="50%" align="center">
      <img src="../screenshots/library-epic-style.webp" alt="Biblioteca no estilo Epic Games"><br>
      <sub><b>Estilo Epic Games</b>: cartões grandes e filtros no topo</sub>
    </td>
  </tr>
  <tr>
    <td width="50%" align="center">
      <img src="../screenshots/game-details.webp" alt="Detalhes do jogo"><br>
      <sub><b>Detalhes do jogo</b>: jogar, favoritar, abrir pasta, capa personalizada</sub>
    </td>
    <td width="50%" align="center">
      <img src="../screenshots/settings-design.webp" alt="Configurações de design"><br>
      <sub><b>Configurações de design</b>: 3 layouts, 8 temas e cor de destaque</sub>
    </td>
  </tr>
</table>

## Instalação

**Requisitos:** Windows 10 ou 11 (64 bits) com [Microsoft WebView2](https://developer.microsoft.com/microsoft-edge/webview2/), que já vem instalado nas versões atuais do Windows.

1. Baixe o **`MAZ-Launcher.exe`** mais recente em [**Releases**](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases/latest).
2. Dê um **clique duplo** no arquivo.
3. Clique em **“Installieren”** (*Instalar*) no menu de instalação.
4. Pronto! Clique em **“Jetzt öffnen”** (*Abrir agora*). Depois disso, abra o launcher pelo atalho da área de trabalho.

> [!TIP]
> **O Windows mostra “O Windows protegeu o computador”?**
> Isso acontece com programas novos sem um certificado de assinatura pago. Clique em **Mais informações** e depois em **Executar assim mesmo**.

> [!TIP]
> **Prefere a versão portátil?** Clique em **“Ohne Installation starten”** (*Iniciar sem instalar*) no menu de instalação ou coloque um arquivo vazio chamado `portable.txt` ao lado do EXE.

**`MAZ-Launcher-mini.exe`** é o mesmo app compactado com UPX. Alguns antivírus acusam EXEs compactados por engano; nesse caso, use a versão normal.

**Desinstalar:** Configurações do Windows → Aplicativos → “MAZ Launcher” → Desinstalar, ou no launcher em **Einstellungen → Allgemein** (*Configurações → Geral*). Seus jogos não são afetados.

## Como usar

- **Jogar:** passe o mouse sobre um jogo e clique em ▶. Um clique na capa abre os detalhes.
- **Adicionar um jogo próprio:** clique em **“Spiel hinzufügen”** (*Adicionar jogo*) à esquerda → **“Durchsuchen…”** (*Procurar…*) ao lado de “Programm” → escolha o executável do jogo → **“Hinzufügen”** (*Adicionar*).
- **Mudar a aparência:** **Einstellungen → Design** → escolha um layout (MAZ / Steam / Epic) e um tema de cores.
- **Ver todos os jogos da Steam, inclusive os não instalados:** **Einstellungen → Verbindungen** (*Conexões*) → informe uma chave gratuita da Steam Web API. As instruções aparecem logo ao lado.
- **Personalizar a checklist:** **Einstellungen → Eigene Spiele** (*Jogos próprios*) → adicione, edite ou remova itens, de forma geral ou por jogo.

### Erros que o launcher explica automaticamente

Arquivo do jogo ausente · unidade ou disco USB desconectado · permissão de administrador necessária · acesso negado · arquivo errado · bloqueado pelo antivírus · bloqueado pelo Windows / Smart App Control · arquivo em uso · DLLs ausentes (Visual C++ / DirectX) · erro **0xc000007b** · **0xc0000142** · travamento logo após abrir · Steam ou Epic não instalados · Steam ou Epic não respondem · chave da Steam API inválida · WebView2 ausente · falha na instalação.

## Compilar a partir do código-fonte

**Requisitos:** [Rust](https://rustup.rs) (stable, 1.85 ou mais recente) no Windows. Só isso: sem Node.js e sem a CLI do Dioxus.

```powershell
git clone https://github.com/MikeAdamRGB/MAZ-Lauchner.git
cd MAZ-Lauchner
cargo run                 # desenvolvimento
cargo build --release     # EXE final: target\release\MAZ-Launcher.exe
.\scripts\build.ps1 -Mini # copia o EXE para dist\ e cria uma versão mini compactada com UPX
```

A folha de estilos compilada (`assets/maz.css`) já está no repositório. Se você alterar classes do Tailwind, rode `.\scripts\build-css.ps1`.

Detalhes técnicos (stack, estrutura do projeto, otimização de tamanho): veja o [README em inglês](../../README.md#tech-stack).

## Perguntas frequentes

**O launcher baixa jogos ou burla proteção contra cópia?**
Não. Jogos da Steam e da Epic são sempre iniciados pelos launchers oficiais. Nenhuma senha é solicitada.

**Por que não vejo todos os meus jogos da Epic?**
A Epic só fornece a lista de jogos comprados após o login. O MAZ Launcher mostra todos os jogos Epic **instalados**.

**A janela fica branca ou não abre.**
O launcher precisa do Microsoft WebView2 (normalmente incluído no Windows 10 e 11). Se estiver faltando, o launcher explica ao iniciar como instalá-lo.

**Meus dados são enviados para algum lugar?**
Sem contas e sem telemetria. A rede só é usada para as capas e, se você adicionar uma chave, para a Steam Web API.

## Contribuindo

Relatos de bugs, ideias e pull requests são bem-vindos: [abrir uma issue](https://github.com/MikeAdamRGB/MAZ-Lauchner/issues/new/choose) · [CONTRIBUTING.md](../../CONTRIBUTING.md)


## Extensões e Minecraft

**Minecraft e NoRisk Client:** o Minecraft Launcher oficial e cada perfil do NoRisk aparecem como blocos próprios. Na aba "Minecraft" você instala mods, pacotes de texturas, shaders e pacotes de dados do Modrinth e do CurseForge.

**Extensões** (desativadas por padrão): pequenos programas externos que trazem novas fontes de jogos, informações ou botões. Antes de instalar você vê todas as permissões e o checksum. Aviso honesto: elas rodam com suas permissões de usuário – instale só as confiáveis. Detalhes: [docs/EXTENSIONS.md](../EXTENSIONS.md).

## Licença

Distribuído sob a [licença MIT](../../LICENSE) © MikeAdamRGB.

Design de componentes inspirado em [rust-ui](https://github.com/rust-ui/ui) (MIT).
Steam é uma marca da Valve Corporation. Epic Games é uma marca da Epic Games, Inc. Este projeto é independente e não tem vínculo com nenhuma das duas empresas.
