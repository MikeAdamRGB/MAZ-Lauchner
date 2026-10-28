<div align="center">

<img src="../../assets/icon-256.png" width="128" alt="Open Game Launcher logosu">

# Open Game Launcher

**Tüm oyunların. Tek bir launcher.**

Steam, Epic Games ve kendi oyunların; tamamen Rust ile yazılmış hızlı, hafif ve şık bir Windows uygulamasında.

[![Son sürüm](https://img.shields.io/github/v/release/MikeAdamRGB/Open-Game-Launcher?label=S%C3%BCr%C3%BCm&color=8b5cf6)](https://github.com/MikeAdamRGB/Open-Game-Launcher/releases/latest)
[![İndirmeler](https://img.shields.io/github/downloads/MikeAdamRGB/Open-Game-Launcher/total?label=%C4%B0ndirme&color=ec4899)](https://github.com/MikeAdamRGB/Open-Game-Launcher/releases)
[![Build](https://github.com/MikeAdamRGB/Open-Game-Launcher/actions/workflows/build.yml/badge.svg)](https://github.com/MikeAdamRGB/Open-Game-Launcher/actions/workflows/build.yml)
![Windows](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6?logo=windows&logoColor=white)
![Rust](https://img.shields.io/badge/100%25-Rust-DEA584?logo=rust&logoColor=white)
[![Lisans: MIT](https://img.shields.io/badge/Lisans-MIT-22c55e)](../../LICENSE)

[English](../../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Italiano](README.it.md) · [Português](README.pt-BR.md) · [Polski](README.pl.md) · **Türkçe** · [Русский](README.ru.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [简体中文](README.zh-CN.md)

[**⬇️ Windows için indir**](https://github.com/MikeAdamRGB/Open-Game-Launcher/releases/latest)

<br>

<img src="../screenshots/library.webp" alt="Open Game Launcher" width="900">

</div>

---

> [!NOTE]
> Uygulamanın arayüzü şu anda **Almanca**. Aşağıda geçen etiketler (örneğin **„Spiel hinzufügen“**) uygulamada göründüğü şekliyle, çevirisiyle birlikte verilmiştir.

## Özellikler

| | |
|---|---|
| 🎮 **Steam** | Tüm disklerdeki yüklü Steam oyunlarını otomatik olarak bulur. İsteğe bağlı bir Steam Web API anahtarıyla, oynama süresi dahil sahip olduğun **tüm** oyunları gösterir. |
| 🟦 **Epic Games** | Yüklü tüm Epic oyunlarını otomatik olarak bulur. |
| 🚀 **Kendi oyunların** | Her oyunu ekleyebilirsin: `.exe`, kısayol veya batch dosyası. **Her** başlatmadan önce bir kontrol listesi sorar: *“Önemli olan her şeyi kapattın mı?”*. **Hayır** dersen oyun başlamaz. |
| 🖼️ **Orijinal simgeler ve kapaklar** | Gerçek oyun simgeleri (doğrudan EXE'den veya Steam'den) ve orijinal kapak görselleri. Her oyuna özel kapak atayabilirsin. |
| 🎨 **3 kütüphane görünümü** | **Steam kütüphanesi gibi**, **Epic Games gibi** ya da launcher'ın kendi **OG tarzında**. |
| 🌈 **8 renk teması** | OG Mor, Gece Yarısı, Neon, Kor, Orman, Sakura, OLED Siyah ve Açık, ayrıca özel vurgu rengi. |
| 🛟 **Hatalar sade bir dille** | Bir şey açılmazsa launcher ne olduğunu ve nasıl düzeltileceğini **adım adım** anlatır; doğrudan yardımcı olan düğmelerle (örneğin *Yönetici olarak çalıştır* veya *Visual C++ indir*). |
| 📦 **Dahili kurulum** | Tek bir EXE: çift tıklayınca masaüstü kısayolu, Başlat menüsü girişi, *Şimdi aç* ve *Uygulama klasörünü aç* seçenekleriyle kurulum menüsü açılır. Yönetici yetkisi gerekmez. |
| 🪶 **Çok küçük** | Boyut için optimize edilmiş tek bir EXE, ek dosya yok. |

## Ekran görüntüleri

<table>
  <tr>
    <td width="50%" align="center">
      <img src="../screenshots/library.webp" alt="Kütüphane"><br>
      <sub><b>Kütüphane</b>: tüm oyunların tek yerde</sub>
    </td>
    <td width="50%" align="center">
      <img src="../screenshots/game-details.webp" alt="Oyun ayrıntıları"><br>
      <sub><b>Oyun ayrıntıları</b>: fragman, bilgi ve başarımlar</sub>
    </td>
  </tr>
  <tr>
    <td width="50%" align="center">
      <img src="../screenshots/minecraft.webp" alt="Minecraft"><br>
      <sub><b>Minecraft</b>: tek tıkla mod</sub>
    </td>
    <td width="50%" align="center">
      <img src="../screenshots/settings-design.webp" alt="Tasarımlar"><br>
      <sub><b>Tasarımlar</b>: 20 hazır tasarım ve kendi tasarımların</sub>
    </td>
  </tr>
</table>

## Kurulum

**Gereksinimler:** [Microsoft WebView2](https://developer.microsoft.com/microsoft-edge/webview2/) yüklü Windows 10 veya 11 (64 bit). WebView2 güncel Windows sürümlerinde hazır gelir.

1. En son **`Open-Game-Launcher.exe`** dosyasını [**Releases**](https://github.com/MikeAdamRGB/Open-Game-Launcher/releases/latest) sayfasından indir.
2. Dosyaya **çift tıkla**.
3. Kurulum menüsünde **„Installieren“** (*Yükle*) düğmesine tıkla.
4. Bitti! **„Jetzt öffnen“** (*Şimdi aç*) düğmesine tıkla. Bundan sonra launcher'ı masaüstü kısayolundan aç.

> [!TIP]
> **Windows “Windows bilgisayarınızı korudu” mu diyor?**
> Bu, ücretli imza sertifikası olmayan yeni programlarda olur. **Ek bilgi**'ye, ardından **Yine de çalıştır**'a tıkla.

> [!TIP]
> **Taşınabilir sürüm mü istiyorsun?** Kurulum menüsünde **„Ohne Installation starten“** (*Kurmadan başlat*) düğmesine tıkla ya da EXE'nin yanına `portable.txt` adlı boş bir dosya koy.


**Kaldırma:** Windows Ayarları → Uygulamalar → “Open Game Launcher” → Kaldır, veya launcher'da **Einstellungen → Allgemein** (*Ayarlar → Genel*). Oyunlarına dokunulmaz.

## Kullanım

- **Oynamak:** farenle bir oyunun üzerine gel ve ▶ simgesine tıkla. Kapağa tıklamak ayrıntıları açar.
- **Kendi oyununu eklemek:** soldaki **„Spiel hinzufügen“** (*Oyun ekle*) → “Programm” yanındaki **„Durchsuchen…“** (*Gözat…*) → oyunun çalıştırılabilir dosyasını seç → **„Hinzufügen“** (*Ekle*).
- **Görünümü değiştirmek:** **Einstellungen → Design** → bir görünüm (OG / Steam / Epic) ve renk teması seç.
- **Yüklü olmayanlar dahil tüm Steam oyunlarını görmek:** **Einstellungen → Verbindungen** (*Bağlantılar*) → ücretsiz bir Steam Web API anahtarı gir. Talimatlar hemen yanında.
- **Kontrol listesini özelleştirmek:** **Einstellungen → Eigene Spiele** (*Kendi oyunlar*) → maddeleri genel olarak veya oyun bazında ekle, düzenle ya da sil.

### Launcher'ın otomatik açıkladığı hatalar

Oyun dosyası eksik · sürücü veya USB disk takılı değil · yönetici yetkisi gerekli · erişim reddedildi · yanlış dosya seçildi · antivirüs tarafından engellendi · Windows / Smart App Control tarafından engellendi · dosya kullanımda · eksik DLL'ler (Visual C++ / DirectX) · **0xc000007b** hatası · **0xc0000142** · başlatmadan hemen sonra çökme · Steam veya Epic yüklü değil · Steam veya Epic yanıt vermiyor · geçersiz Steam API anahtarı · WebView2 eksik · kurulum başarısız.

## Kaynaktan derleme

**Gereksinimler:** Windows'ta [Rust](https://rustup.rs) (stable, 1.85 veya üzeri). Başka bir şey gerekmez: Node.js ve Dioxus CLI yok.

```powershell
git clone https://github.com/MikeAdamRGB/Open-Game-Launcher.git
cd MAZ-Lauchner
cargo run                 # geliştirme
cargo build --release     # son EXE: target\release\Open-Game-Launcher.exe
.\scripts\build.ps1 -Mini # EXE'yi dist\ klasörüne kopyalar ve UPX ile sıkıştırılmış mini sürüm oluşturur
```

Derlenmiş stil dosyası (`assets/maz.css`) zaten depoda bulunur. Tailwind sınıflarını değiştirirsen `.\scripts\build-css.ps1` komutunu çalıştır.

Teknik ayrıntılar (teknoloji yığını, proje yapısı, boyut optimizasyonu) için [İngilizce README](../../README.md#tech-stack)'ye bak.

## SSS

**Launcher oyun indiriyor veya kopya korumasını aşıyor mu?**
Hayır. Steam ve Epic oyunları her zaman resmi launcher'ları üzerinden başlatılır. Hiçbir zaman şifre istenmez.

**Neden tüm Epic oyunlarımı göremiyorum?**
Epic, satın alınan oyunların listesini yalnızca giriş yaptıktan sonra verir. Open Game Launcher tüm **yüklü** Epic oyunlarını gösterir.

**Pencere beyaz kalıyor veya açılmıyor.**
Launcher, Microsoft WebView2'ye ihtiyaç duyar (Windows 10 ve 11'de normalde bulunur). Eksikse launcher açılışta nasıl kurulacağını anlatır.

**Verilerim bir yere gönderiliyor mu?**
Hesap yok, telemetri yok. İnternet yalnızca kapak görselleri ve anahtar eklediysen Steam Web API için kullanılır.

## Katkıda bulunma

Hata bildirimleri, fikirler ve pull request'ler memnuniyetle karşılanır: [issue aç](https://github.com/MikeAdamRGB/Open-Game-Launcher/issues/new/choose) · [CONTRIBUTING.md](../../CONTRIBUTING.md)


## Uzantılar ve Minecraft

**Minecraft ve NoRisk Client:** Resmi Minecraft Launcher ve her NoRisk profili ayrı kutucuk olarak görünür. "Minecraft" sekmesinden Modrinth ve CurseForge'dan mod, doku paketi, shader ve veri paketi yüklersin.

**Uzantılar** (varsayılan olarak kapalı): yeni oyun kaynakları, bilgiler veya düğmeler ekleyen küçük harici programlar. Yüklemeden önce tüm izinleri ve sağlama toplamını görürsün. Dürüst not: kullanıcı haklarınla çalışırlar – yalnızca güvendiklerini yükle. Ayrıntılar: [docs/EXTENSIONS.md](../EXTENSIONS.md).

## Lisans

[MIT Lisansı](../../LICENSE) ile yayınlanmıştır © MikeAdamRGB.

Bileşen tasarımı [rust-ui](https://github.com/rust-ui/ui)'dan (MIT) esinlenmiştir.
Steam, Valve Corporation'ın ticari markasıdır. Epic Games, Epic Games, Inc.'in ticari markasıdır. Bu proje bağımsızdır ve iki şirketle de bağlantılı değildir.
