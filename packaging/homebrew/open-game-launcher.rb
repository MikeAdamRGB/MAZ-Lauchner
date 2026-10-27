cask "open-game-launcher" do
  version "{VERSION}"
  sha256 "{SHA_Open-Game-Launcher-{VERSION}-macOS.dmg}"

  url "https://github.com/MikeAdamRGB/Open-Game-Launcher/releases/download/v#{version}/Open-Game-Launcher-#{version}-macOS.dmg"
  name "Open Game Launcher"
  desc "All your Steam, Epic, GOG and own games in one launcher"
  homepage "https://github.com/MikeAdamRGB/Open-Game-Launcher"

  livecheck do
    url :url
    strategy :github_latest
  end

  depends_on macos: ">= :big_sur"

  app "Open Game Launcher.app"

  zap trash: [
    "~/Library/Application Support/Open Game Launcher",
    "~/Library/Caches/Open Game Launcher",
  ]
end
