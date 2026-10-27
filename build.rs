//! Bettet Icon, Versionsinfos und Windows-Manifest in die EXE ein.

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=assets/maz.css");
    println!("cargo:rerun-if-changed=build.rs");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    // SICHERHEIT: DLLs, die die EXE beim Start braucht, nur aus System32 laden – nie aus dem
    // Ordner, in dem die EXE liegt (z. B. "Downloads", wo jemand eine falsche DLL ablegen könnte).
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        println!("cargo:rustc-link-arg-bins=/DEPENDENTLOADFLAG:0x800");
        // Schattenstapel (Intel CET / AMD Shadow Stack): schützt Rücksprungadressen
        if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("x86_64") {
            println!("cargo:rustc-link-arg-bins=/CETCOMPAT");
        }
    }

    let version = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "1.0.0".into());
    let mut res = winresource::WindowsResource::new();
    // ID 1 = Icon im Explorer, 32512 = Fenster-/Taskleisten-Icon (wird von Dioxus genutzt)
    res.set_icon("assets/icon.ico");
    res.set_icon_with_id("assets/icon.ico", "32512");
    res.set("ProductName", "Open Game Launcher");
    res.set("FileDescription", "Open Game Launcher");
    res.set("CompanyName", "MikeAdamRGB");
    res.set("LegalCopyright", "MIT-Lizenz");
    res.set("OriginalFilename", "Open-Game-Launcher.exe");
    res.set("ProductVersion", &version);
    res.set("FileVersion", &version);
    res.set_manifest(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity type="win32" name="MikeAdamRGB.OpenGameLauncher" version="1.0.0.0"/>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
  <dependency>
    <dependentAssembly>
      <assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0"
        processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*"/>
    </dependentAssembly>
  </dependency>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
    </application>
  </compatibility>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
      <longPathAware xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">true</longPathAware>
    </windowsSettings>
  </application>
</assembly>"#,
    );
    if let Err(e) = res.compile() {
        // Kein Abbruch: Ohne Icon läuft die EXE trotzdem.
        println!("cargo:warning=Ressourcen (Icon/Manifest) konnten nicht eingebettet werden: {e}");
    }
}
