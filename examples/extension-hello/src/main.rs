//! Beispiel-Erweiterung für den Open Game Launcher: zwei Test-Spiele, Infos und eine Aktion.
use og_extension_sdk::*;

struct Hello {
    dir: String,
}

impl Extension for Hello {
    fn hello(&mut self, _h: &og_extension_sdk::Hello) -> HelloReply {
        HelloReply { name: "Hello Extension".into(), version: "1.0.0".into(), kinds: vec!["library".into(), "metadata".into(), "action".into()] }
    }

    fn games(&mut self) -> Result<Vec<GameInfo>, ExtError> {
        Ok(vec![
            GameInfo {
                id: "hello-1".into(),
                name: "Hello Game One".into(),
                // Spielordner = Ordner der Erweiterung (nur für das Beispiel)
                install_dir: Some(self.dir.clone()),
                launch: Launch { uri: Some("https://github.com/MikeAdamRGB/Open-Game-Launcher".into()), ..Default::default() },
                cover: Some("cover.png".into()),
                ..Default::default()
            },
            GameInfo {
                id: "hello-2".into(),
                name: "Hello Game Two".into(),
                launch: Launch { uri: Some("https://mikeadamrgb.github.io/Open-Game-Launcher/".into()), ..Default::default() },
                ..Default::default()
            },
        ])
    }

    fn metadata(&mut self, name: &str, _id: Option<&str>) -> Result<Option<Metadata>, ExtError> {
        if !name.starts_with("Hello Game") {
            return Ok(None);
        }
        Ok(Some(Metadata {
            description: format!("{name} ist ein Test-Spiel aus der Beispiel-Erweiterung."),
            genres: vec!["Demo".into()],
            developer: "OG".into(),
            ..Default::default()
        }))
    }

    fn actions(&mut self, game: &GameRef) -> Result<Vec<Action>, ExtError> {
        if game.install_dir.is_some() {
            Ok(vec![Action { id: "open-folder".into(), label: "@open_folder".into() }])
        } else {
            Ok(vec![Action { id: "hello".into(), label: "@say_hello".into() }])
        }
    }

    fn run_action(&mut self, action: &str, game: &GameRef) -> Result<ActionResult, ExtError> {
        match action {
            "open-folder" => Ok(ActionResult::OpenFolder { path: game.install_dir.clone().unwrap_or_default() }),
            "hello" => Ok(ActionResult::Message { text: format!("Hallo von {}!", game.name) }),
            _ => Err(ExtError::unsupported()),
        }
    }
}

fn main() {
    let dir = std::env::var("OG_EXTENSION_DIR").or_else(|_| std::env::var("MAZ_EXTENSION_DIR")).unwrap_or_else(|_| ".".into());
    run(Hello { dir })
}
