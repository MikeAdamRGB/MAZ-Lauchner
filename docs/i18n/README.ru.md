<div align="center">

<img src="../../assets/icon-256.png" width="128" alt="Логотип MAZ Launcher">

# MAZ Launcher

**Все твои игры. Один лаунчер.**

Steam, Epic Games и твои собственные игры в одном быстром, лёгком и красивом приложении для Windows, написанном на 100% на Rust.

[![Последняя версия](https://img.shields.io/github/v/release/MikeAdamRGB/MAZ-Lauchner?label=%D0%92%D0%B5%D1%80%D1%81%D0%B8%D1%8F&color=8b5cf6)](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases/latest)
[![Загрузки](https://img.shields.io/github/downloads/MikeAdamRGB/MAZ-Lauchner/total?label=%D0%97%D0%B0%D0%B3%D1%80%D1%83%D0%B7%D0%BA%D0%B8&color=ec4899)](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases)
[![Build](https://github.com/MikeAdamRGB/MAZ-Lauchner/actions/workflows/build.yml/badge.svg)](https://github.com/MikeAdamRGB/MAZ-Lauchner/actions/workflows/build.yml)
![Windows](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6?logo=windows&logoColor=white)
![Rust](https://img.shields.io/badge/100%25-Rust-DEA584?logo=rust&logoColor=white)
[![Лицензия: MIT](https://img.shields.io/badge/License-MIT-22c55e)](../../LICENSE)

[English](../../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Italiano](README.it.md) · [Português](README.pt-BR.md) · [Polski](README.pl.md) · [Türkçe](README.tr.md) · **Русский** · [日本語](README.ja.md) · [한국어](README.ko.md) · [简体中文](README.zh-CN.md)

[**⬇️ Скачать для Windows**](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases/latest)

<br>

<img src="../screenshots/library-epic-style.webp" alt="Библиотека MAZ Launcher в стиле Epic Games" width="900">

</div>

---

> [!NOTE]
> Сейчас интерфейс приложения доступен на **немецком** языке. Названия кнопок ниже (например, **«Spiel hinzufügen»**) приведены так, как они выглядят в приложении, вместе с переводом.

## Возможности

| | |
|---|---|
| 🎮 **Steam** | Автоматически находит все установленные игры Steam на всех дисках. С необязательным ключом Steam Web API показывает **все** купленные игры вместе с игровым временем. |
| 🟦 **Epic Games** | Автоматически находит все установленные игры Epic. |
| 🚀 **Свои игры** | Добавь любую игру: `.exe`, ярлык или batch-файл. Перед **каждым** запуском чек-лист спрашивает: *«Ты отключил всё важное?»*. Если ответить **Нет**, игра не запустится. |
| 🖼️ **Оригинальные иконки и обложки** | Настоящие иконки (прямо из EXE или из Steam) и официальные обложки. Для любой игры можно задать свою обложку. |
| 🎨 **3 вида библиотеки** | Выглядит **как библиотека Steam**, **как Epic Games** или в собственном **стиле MAZ**. |
| 🌈 **8 цветовых тем** | MAZ Фиолетовый, Полночь, Неон, Угли, Лес, Сакура, OLED Чёрный и Светлая, плюс свой акцентный цвет. |
| 🛟 **Ошибки простыми словами** | Если что-то не запускается, лаунчер **шаг за шагом** объясняет, что случилось и как это исправить, с кнопками, которые сразу помогают (например, *Запустить от имени администратора* или *Скачать Visual C++*). |
| 📦 **Встроенный установщик** | Один EXE-файл: двойной щелчок открывает меню установки с ярлыком на рабочем столе, пунктом в меню «Пуск», кнопками *Открыть сейчас* и *Открыть папку приложения*. Права администратора не нужны. |
| 🪶 **Крошечный размер** | Один оптимизированный по размеру EXE без дополнительных файлов. |

## Скриншоты

<table>
  <tr>
    <td width="50%" align="center">
      <img src="../screenshots/library-steam-style.webp" alt="Библиотека в стиле Steam"><br>
      <sub><b>Стиль Steam</b>: список игр слева, «Недавние» и сетка всех игр</sub>
    </td>
    <td width="50%" align="center">
      <img src="../screenshots/library-epic-style.webp" alt="Библиотека в стиле Epic Games"><br>
      <sub><b>Стиль Epic Games</b>: большие карточки и фильтры сверху</sub>
    </td>
  </tr>
  <tr>
    <td width="50%" align="center">
      <img src="../screenshots/game-details.webp" alt="Информация об игре"><br>
      <sub><b>Карточка игры</b>: играть, в избранное, открыть папку, своя обложка</sub>
    </td>
    <td width="50%" align="center">
      <img src="../screenshots/settings-design.webp" alt="Настройки оформления"><br>
      <sub><b>Настройки оформления</b>: 3 вида, 8 тем и акцентный цвет</sub>
    </td>
  </tr>
</table>

## Установка

**Требования:** Windows 10 или 11 (64-бит) с [Microsoft WebView2](https://developer.microsoft.com/microsoft-edge/webview2/), который предустановлен в актуальных версиях Windows.

1. Скачай последнюю версию **`MAZ-Launcher.exe`** на странице [**Releases**](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases/latest).
2. Запусти файл **двойным щелчком**.
3. В меню установки нажми **«Installieren»** (*Установить*).
4. Готово! Нажми **«Jetzt öffnen»** (*Открыть сейчас*). Дальше запускай лаунчер через ярлык на рабочем столе.

> [!TIP]
> **Windows пишет «Система Windows защитила ваш компьютер»?**
> Так бывает с новыми программами без платного сертификата подписи. Нажми **Подробнее**, затем **Выполнить в любом случае**.

> [!TIP]
> **Нужна портативная версия?** В меню установки нажми **«Ohne Installation starten»** (*Запустить без установки*) или положи рядом с EXE пустой файл `portable.txt`.

**`MAZ-Launcher-mini.exe`** — то же приложение, упакованное UPX. Некоторые антивирусы ошибочно реагируют на упакованные EXE; в таком случае используй обычную версию.

**Удаление:** Параметры Windows → Приложения → «MAZ Launcher» → Удалить, или в лаунчере в разделе **Einstellungen → Allgemein** (*Настройки → Общие*). Твои игры не затрагиваются.

## Как пользоваться

- **Играть:** наведи курсор на игру и нажми ▶. Щелчок по обложке открывает подробности.
- **Добавить свою игру:** слева нажми **«Spiel hinzufügen»** (*Добавить игру*) → **«Durchsuchen…»** (*Обзор…*) рядом с «Programm» → выбери исполняемый файл игры → **«Hinzufügen»** (*Добавить*).
- **Изменить внешний вид:** **Einstellungen → Design** → выбери вид (MAZ / Steam / Epic) и цветовую тему.
- **Видеть все игры Steam, даже неустановленные:** **Einstellungen → Verbindungen** (*Подключения*) → введи бесплатный ключ Steam Web API. Инструкция находится прямо рядом.
- **Настроить чек-лист:** **Einstellungen → Eigene Spiele** (*Свои игры*) → добавляй, изменяй или удаляй пункты, для всех игр или для конкретной.

### Ошибки, которые лаунчер объясняет автоматически

Файл игры отсутствует · диск или USB-накопитель не подключён · нужны права администратора · доступ запрещён · выбран не тот файл · заблокировано антивирусом · заблокировано Windows / Smart App Control · файл занят · отсутствуют DLL (Visual C++ / DirectX) · ошибка **0xc000007b** · **0xc0000142** · вылет сразу после запуска · Steam или Epic не установлены · Steam или Epic не отвечают · неверный ключ Steam API · отсутствует WebView2 · ошибка установки.

## Сборка из исходников

**Требования:** [Rust](https://rustup.rs) (stable, 1.85 или новее) на Windows. Больше ничего: ни Node.js, ни Dioxus CLI.

```powershell
git clone https://github.com/MikeAdamRGB/MAZ-Lauchner.git
cd MAZ-Lauchner
cargo run                 # разработка
cargo build --release     # готовый EXE: target\release\MAZ-Launcher.exe
.\scripts\build.ps1 -Mini # копирует EXE в dist\ и собирает mini-версию, упакованную UPX
```

Скомпилированный CSS (`assets/maz.css`) уже лежит в репозитории. Если меняешь классы Tailwind, запусти `.\scripts\build-css.ps1`.

Технические подробности (стек, структура проекта, оптимизация размера) — в [README на английском](../../README.md#tech-stack).

## FAQ

**Лаунчер скачивает игры или обходит защиту от копирования?**
Нет. Игры Steam и Epic всегда запускаются через официальные лаунчеры. Пароли никогда не запрашиваются.

**Почему я вижу не все свои игры Epic?**
Epic отдаёт список купленных игр только после входа в аккаунт. MAZ Launcher показывает все **установленные** игры Epic.

**Окно остаётся белым или не открывается.**
Лаунчеру нужен Microsoft WebView2 (обычно входит в Windows 10 и 11). Если его нет, лаунчер при запуске объяснит, как его установить.

**Отправляются ли мои данные куда-либо?**
Никаких аккаунтов и телеметрии. Сеть используется только для обложек и, если ты добавишь ключ, для Steam Web API.

## Участие в разработке

Сообщения об ошибках, идеи и pull request'ы приветствуются: [создать issue](https://github.com/MikeAdamRGB/MAZ-Lauchner/issues/new/choose) · [CONTRIBUTING.md](../../CONTRIBUTING.md)


## Расширения и Minecraft

**Minecraft и NoRisk Client:** официальный Minecraft Launcher и каждый профиль NoRisk – отдельные плитки. На вкладке «Minecraft» можно ставить моды, ресурспаки, шейдеры и датапаки с Modrinth и CurseForge.

**Расширения** (по умолчанию выключены): небольшие внешние программы с новыми источниками игр, информацией или кнопками. Перед установкой видны все права и контрольная сумма. Честно: они работают с твоими правами – ставь только те, которым доверяешь. Подробности: [docs/EXTENSIONS.md](../EXTENSIONS.md).

## Лицензия

Распространяется под [лицензией MIT](../../LICENSE) © MikeAdamRGB.

Дизайн компонентов вдохновлён [rust-ui](https://github.com/rust-ui/ui) (MIT).
Steam — товарный знак Valve Corporation. Epic Games — товарный знак Epic Games, Inc. Этот проект независим и не связан ни с одной из этих компаний.
