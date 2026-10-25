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
            let app_ids: Vec<String> = k.enum_keys().flatten().filter(|a| !a.eq_ignore_ascii_case("Capabilities")).collect();
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

/// Paket mit diesem Namen (ohne Version) suchen
pub fn find(name: &str) -> Option<Package> {
    packages().into_iter().find(|p| p.name.eq_ignore_ascii_case(name))
}

/// App starten (wie aus dem Startmenü)
pub fn launch(family: &str, app_id: &str) -> Result<(), String> {
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
    }
}
