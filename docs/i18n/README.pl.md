<div align="center">

<img src="../../assets/icon-256.png" width="128" alt="Logo MAZ Launcher">

# MAZ Launcher

**Wszystkie Twoje gry. Jeden launcher.**

Steam, Epic Games i Twoje własne gry w jednej szybkiej, lekkiej i ładnej aplikacji dla Windows, napisanej w 100% w Rust.

[![Najnowsza wersja](https://img.shields.io/github/v/release/MikeAdamRGB/MAZ-Lauchner?label=Wersja&color=8b5cf6)](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases/latest)
[![Pobrania](https://img.shields.io/github/downloads/MikeAdamRGB/MAZ-Lauchner/total?label=Pobrania&color=ec4899)](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases)
[![Build](https://github.com/MikeAdamRGB/MAZ-Lauchner/actions/workflows/build.yml/badge.svg)](https://github.com/MikeAdamRGB/MAZ-Lauchner/actions/workflows/build.yml)
![Windows](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6?logo=windows&logoColor=white)
![Rust](https://img.shields.io/badge/100%25-Rust-DEA584?logo=rust&logoColor=white)
[![Licencja: MIT](https://img.shields.io/badge/Licencja-MIT-22c55e)](../../LICENSE)

[English](../../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Italiano](README.it.md) · [Português](README.pt-BR.md) · **Polski** · [Türkçe](README.tr.md) · [Русский](README.ru.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [简体中文](README.zh-CN.md)

[**⬇️ Pobierz dla Windows**](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases/latest)

<br>

<img src="../screenshots/library-epic-style.webp" alt="Biblioteka MAZ Launcher w stylu Epic Games" width="900">

</div>

---

> [!NOTE]
> Interfejs aplikacji jest obecnie dostępny po **niemiecku**. Etykiety cytowane poniżej (na przykład **„Spiel hinzufügen”**) podano dokładnie tak, jak widać je w aplikacji, wraz z tłumaczeniem.

## Funkcje

| | |
|---|---|
| 🎮 **Steam** | Automatycznie wykrywa wszystkie zainstalowane gry Steam na wszystkich dyskach. Z opcjonalnym kluczem Steam Web API pokazuje **wszystkie** posiadane gry wraz z czasem gry. |
| 🟦 **Epic Games** | Automatycznie wykrywa wszystkie zainstalowane gry Epic. |
| 🚀 **Własne gry** | Dodaj dowolną grę: plik `.exe`, skrót lub plik wsadowy. Przed **każdym** uruchomieniem lista kontrolna pyta *„Czy wyłączyłeś wszystko, co ważne?”*. Jeśli odpowiesz **Nie**, gra się nie uruchomi. |
| 🖼️ **Oryginalne ikony i okładki** | Prawdziwe ikony gier (pobrane bezpośrednio z pliku EXE lub ze Steam) i oryginalne okładki. Każdej grze możesz ustawić własną okładkę. |
| 🎨 **3 układy biblioteki** | Wygląda **jak biblioteka Steam**, **jak Epic Games** albo w **stylu MAZ**. |
| 🌈 **8 motywów kolorystycznych** | MAZ Fiolet, Północ, Neon, Żar, Las, Sakura, OLED Czerń i Jasny, a do tego własny kolor akcentu. |
| 🛟 **Błędy wyjaśnione prostym językiem** | Gdy coś się nie uruchamia, launcher **krok po kroku** wyjaśnia, co się stało i jak to naprawić, z przyciskami, które od razu pomagają (np. *Uruchom jako administrator* lub *Pobierz Visual C++*). |
| 📦 **Wbudowany instalator** | Jeden plik EXE: dwukrotne kliknięcie otwiera menu instalacji ze skrótem na pulpicie, wpisem w menu Start, *Otwórz teraz* i *Otwórz folder aplikacji*. Bez uprawnień administratora. |
| 🪶 **Bardzo mały** | Jeden zoptymalizowany pod kątem rozmiaru plik EXE, bez dodatkowych plików. |

## Zrzuty ekranu

<table>
  <tr>
    <td width="50%" align="center">
      <img src="../screenshots/library-steam-style.webp" alt="Biblioteka w stylu Steam"><br>
      <sub><b>Styl Steam</b>: lista gier po lewej, „Ostatnio grane” i siatka wszystkich gier</sub>
    </td>
    <td width="50%" align="center">
      <img src="../screenshots/library-epic-style.webp" alt="Biblioteka w stylu Epic Games"><br>
      <sub><b>Styl Epic Games</b>: duże karty i filtry u góry</sub>
    </td>
  </tr>
  <tr>
    <td width="50%" align="center">
      <img src="../screenshots/game-details.webp" alt="Szczegóły gry"><br>
      <sub><b>Szczegóły gry</b>: graj, ulubione, folder gry, własna okładka</sub>
    </td>
    <td width="50%" align="center">
      <img src="../screenshots/settings-design.webp" alt="Ustawienia wyglądu"><br>
      <sub><b>Ustawienia wyglądu</b>: 3 układy, 8 motywów i kolor akcentu</sub>
    </td>
  </tr>
</table>

## Instalacja

**Wymagania:** Windows 10 lub 11 (64-bit) z [Microsoft WebView2](https://developer.microsoft.com/microsoft-edge/webview2/), który jest preinstalowany w aktualnych wersjach Windows.

1. Pobierz najnowszy **`MAZ-Launcher.exe`** z [**Releases**](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases/latest).
2. Kliknij plik **dwukrotnie**.
3. W menu instalacji kliknij **„Installieren”** (*Zainstaluj*).
4. Gotowe! Kliknij **„Jetzt öffnen”** (*Otwórz teraz*). Od tej pory uruchamiaj launcher skrótem z pulpitu.

> [!TIP]
> **Windows wyświetla „System Windows ochronił ten komputer”?**
> Tak się dzieje przy nowych programach bez płatnego certyfikatu podpisu. Kliknij **Więcej informacji**, a potem **Uruchom mimo to**.

> [!TIP]
> **Wolisz wersję przenośną?** W menu instalacji kliknij **„Ohne Installation starten”** (*Uruchom bez instalacji*) albo umieść obok pliku EXE pusty plik o nazwie `portable.txt`.

**`MAZ-Launcher-mini.exe`** to ta sama aplikacja spakowana UPX. Niektóre antywirusy błędnie zgłaszają spakowane pliki EXE; wtedy użyj zwykłej wersji.

**Odinstalowanie:** Ustawienia Windows → Aplikacje → „MAZ Launcher” → Odinstaluj, albo w launcherze w **Einstellungen → Allgemein** (*Ustawienia → Ogólne*). Twoje gry pozostają nienaruszone.

## Jak używać

- **Granie:** najedź myszą na grę i kliknij ▶. Kliknięcie okładki otwiera szczegóły.
- **Dodawanie własnej gry:** kliknij **„Spiel hinzufügen”** (*Dodaj grę*) po lewej → **„Durchsuchen…”** (*Przeglądaj…*) przy „Programm” → wybierz plik startowy gry → **„Hinzufügen”** (*Dodaj*).
- **Zmiana wyglądu:** **Einstellungen → Design** → wybierz układ (MAZ / Steam / Epic) i motyw kolorystyczny.
- **Wszystkie gry Steam, także niezainstalowane:** **Einstellungen → Verbindungen** (*Połączenia*) → wpisz darmowy klucz Steam Web API. Instrukcja jest tuż obok.
- **Własna lista kontrolna:** **Einstellungen → Eigene Spiele** (*Własne gry*) → dodawaj, edytuj lub usuwaj punkty, ogólnie lub dla konkretnej gry.

### Błędy, które launcher wyjaśnia automatycznie

Brak pliku gry · dysk lub pendrive niepodłączony · wymagane uprawnienia administratora · odmowa dostępu · wybrano zły plik · zablokowane przez antywirus · zablokowane przez Windows / Smart App Control · plik w użyciu · brakujące biblioteki DLL (Visual C++ / DirectX) · błąd **0xc000007b** · **0xc0000142** · awaria tuż po uruchomieniu · brak Steam lub Epic · Steam lub Epic nie odpowiada · nieprawidłowy klucz Steam API · brak WebView2 · instalacja nie powiodła się.

## Kompilacja ze źródeł

**Wymagania:** [Rust](https://rustup.rs) (stable, 1.85 lub nowszy) na Windows. Nic więcej: bez Node.js i bez Dioxus CLI.

```powershell
git clone https://github.com/MikeAdamRGB/MAZ-Lauchner.git
cd MAZ-Lauchner
cargo run                 # tryb deweloperski
cargo build --release     # gotowy EXE: target\release\MAZ-Launcher.exe
.\scripts\build.ps1 -Mini # kopiuje EXE do dist\ i tworzy wersję mini spakowaną UPX
```

Skompilowany arkusz stylów (`assets/maz.css`) jest już w repozytorium. Po zmianie klas Tailwind uruchom `.\scripts\build-css.ps1`.

Szczegóły techniczne (stack, struktura projektu, optymalizacja rozmiaru): zobacz [README po angielsku](../../README.md#tech-stack).

## FAQ

**Czy launcher pobiera gry albo omija zabezpieczenia przed kopiowaniem?**
Nie. Gry ze Steam i Epic są zawsze uruchamiane przez oficjalne launchery. Launcher nigdy nie pyta o hasła.

**Dlaczego nie widzę wszystkich moich gier z Epic?**
Epic udostępnia listę kupionych gier dopiero po zalogowaniu. MAZ Launcher pokazuje wszystkie **zainstalowane** gry Epic.

**Okno pozostaje białe albo się nie otwiera.**
Launcher wymaga Microsoft WebView2 (zwykle dołączony do Windows 10 i 11). Jeśli go brakuje, launcher przy starcie wyjaśnia, jak go zainstalować.

**Czy moje dane są gdzieś wysyłane?**
Bez kont i bez telemetrii. Sieć służy tylko do pobierania okładek oraz, jeśli dodasz klucz, do Steam Web API.

## Współpraca

Zgłoszenia błędów, pomysły i pull requesty są mile widziane: [otwórz issue](https://github.com/MikeAdamRGB/MAZ-Lauchner/issues/new/choose) · [CONTRIBUTING.md](../../CONTRIBUTING.md)


## Rozszerzenia i Minecraft

**Minecraft i NoRisk Client:** oficjalny Minecraft Launcher i każdy profil NoRisk to osobne kafelki. W karcie „Minecraft” zainstalujesz mody, paczki tekstur, shadery i paczki danych z Modrinth i CurseForge.

**Rozszerzenia** (domyślnie wyłączone): małe zewnętrzne programy dodające źródła gier, informacje lub przyciski. Przed instalacją widzisz wszystkie uprawnienia i sumę kontrolną. Uczciwie: działają z twoimi uprawnieniami – instaluj tylko zaufane. Szczegóły: [docs/EXTENSIONS.md](../EXTENSIONS.md).

## Licencja

Udostępnione na [licencji MIT](../../LICENSE) © MikeAdamRGB.

Projekt komponentów inspirowany [rust-ui](https://github.com/rust-ui/ui) (MIT).
Steam jest znakiem towarowym Valve Corporation. Epic Games jest znakiem towarowym Epic Games, Inc. Ten projekt jest niezależny i niepowiązany z żadną z tych firm.
