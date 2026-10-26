//! Microsoft-Store-/Xbox-Apps (AppX/MSIX) unter Windows finden und starten.
//!
//! Installierte Pakete stehen in der Registry unter
//! `HKCU\Software\Classes\Local Settings\Software\Microsoft\Windows\CurrentVersion\AppModel\Repository\Packages\<Paketname>`.
//! Gestartet wird über `explorer.exe shell:AppsFolder\<Familienname>!<App-ID>` – genau wie aus dem Startmenü.

#[derive(Clone, Debug, PartialEq)]
pub struct Package {
    /// z. B. "ROBLOXCORPORATION.ROBLOX_2.640.0.0_x64__55nm5eh3cm0pr"
    pub full_name: String,
    /// z. B. "ROBLOXCORPORATION.ROBLOX_55nm5eh3cm0pr"
    pub family: String,
    /// z. B. "ROBLOXCORPORATION.ROBLOX"
    pub name: String,
    pub display_name: String,
    pub root: String,
    pub app_ids: Vec<String>,
}

/// "Name_Version_Arch_ResourceId_PublisherId" → ("Name", "Name_PublisherId")
pub fn family_of(full_name: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = full_name.split('_').collect();
    if parts.len() < 5 {
        return None;
    }
    let name = parts[0].to_string();
    let publisher = parts[parts.len() - 1];
    if publisher.is_empty() {
        return None;
    }
    Some((name.clone(), format!("{name}_{publisher}")))
}

/// Alle installierten Store-Pakete (nur Windows; sonst leer)
pub fn packages() -> Vec<Package> {
    #[cfg(windows)]
    {
        use winreg::enums::HKEY_CURRENT_USER;
        use winreg::RegKey;
        let path = "Software\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\CurrentVersion\\AppModel\\Repository\\Packages";
        let Ok(root) = RegKey::predef(HKEY_CURRENT_USER).open_subkey(path) else { return Vec::new() };
        let mut out = Vec::new();
        for full in root.enum_keys().flatten() {
            let Some((name, family)) = family_of(&full) else { continue };
            let Ok(k) = root.open_subkey(&full) else { continue };
            let display: String = k.get_value("DisplayName").unwrap_or_default();
            let pkg_root: String = k.get_value("PackageRootFolder").unwrap_or_default();
            // App-Kennungen aus der Paketbeschreibung (verlässlich), sonst aus der Registry
            let mut app_ids = std::fs::read_to_string(std::path::Path::new(&pkg_root).join("AppxManifest.xml"))
                .map(|x| manifest_app_ids(&x))
                .unwrap_or_default();
            if app_ids.is_empty() {
                app_ids = k.enum_keys().flatten().filter(|a| !a.eq_ignore_ascii_case("Capabilities")).collect();
            }
            out.push(Package {
                full_name: full,
                family,
                name,
                // "@{…ms-resource:…}" = Text aus dem Paket, den wir nicht lesen können → leer lassen
                display_name: if display.starts_with('@') { String::new() } else { display },
                root: pkg_root,
                app_ids,
            });
        }
        out
    }
    #[cfg(not(windows))]
    {
        Vec::new()
    }
}

/// `<Application Id="…">` aus einer AppxManifest.xml
pub fn manifest_app_ids(xml: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(i) = rest.find("<Application ") {
        let tag_end = rest[i..].find('>').map(|e| i + e).unwrap_or(rest.len());
        let tag = &rest[i..tag_end];
        if let Some(a) = tag.find(" Id=\"") {
            let v = &tag[a + 5..];
            if let Some(end) = v.find('"') {
                let id = &v[..end];
                if !id.is_empty() && id.len() < 100 && id.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-')) {
                    out.push(id.to_string());
                }
            }
        }
        rest = &rest[tag_end.min(rest.len())..];
        if rest.len() <= 1 {
            break;
        }
    }
    out
}

impl Package {
    /// Richtige App-Kennung: die gewünschte, wenn es sie gibt – sonst die erste App im Paket.
    /// (Eine falsche Kennung lässt Windows nur den Explorer öffnen.)
    pub fn app_id(&self, preferred: &str) -> Option<String> {
        self.app_ids
            .iter()
            .find(|a| a.eq_ignore_ascii_case(preferred))
            .or_else(|| self.app_ids.first())
            .cloned()
    }
}

/// Paket mit diesem Namen (ohne Version) suchen – bevorzugt eins, dessen Ordner existiert
pub fn find(name: &str) -> Option<Package> {
    let mut list: Vec<Package> = packages().into_iter().filter(|p| p.name.eq_ignore_ascii_case(name)).collect();
    list.sort_by_key(|p| !std::path::Path::new(&p.root).is_dir());
    list.into_iter().next()
}

/// App starten (wie aus dem Startmenü)
pub fn launch(family: &str, app_id: &str) -> Result<(), String> {
    let simple = |v: &str| !v.is_empty() && v.len() < 130 && v.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
    if !simple(family) || !simple(app_id) {
        return Err("Invalid app id".into());
    }
    #[cfg(windows)]
    {
        crate::sys::quiet_command(crate::sys::system_program("explorer.exe"))
            .arg(format!("shell:AppsFolder\\{family}!{app_id}"))
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    #[cfg(not(windows))]
    {
        let _ = (family, app_id);
        Err("Microsoft Store apps only exist on Windows".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn family_names() {
        assert_eq!(
            family_of("ROBLOXCORPORATION.ROBLOX_2.640.0.0_x64__55nm5eh3cm0pr"),
            Some(("ROBLOXCORPORATION.ROBLOX".into(), "ROBLOXCORPORATION.ROBLOX_55nm5eh3cm0pr".into()))
        );
        assert_eq!(family_of("Kaputt"), None);
        let xml = r#"<Package><Applications><Application Id="Minecraft" Executable="Minecraft.exe" EntryPoint="x"><uap:VisualElements/></Application><Application Id="Evil&quot;" /></Applications></Package>"#;
        assert_eq!(manifest_app_ids(xml), vec!["Minecraft".to_string()]);
        let p = Package { full_name: String::new(), family: "F".into(), name: "N".into(), display_name: String::new(), root: String::new(), app_ids: vec!["Minecraft".into()] };
        assert_eq!(p.app_id("Game").as_deref(), Some("Minecraft"));
        assert_eq!(p.app_id("minecraft").as_deref(), Some("Minecraft"));
    }
}
