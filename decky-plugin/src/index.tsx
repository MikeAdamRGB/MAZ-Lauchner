// Open Game Launcher – Decky-Plugin (Schnellzugriffsmenü auf dem Steam Deck)
//
// Zeigt die Spiele aus dem Open Game Launcher und startet sie über EINE Steam-Verknüpfung
// („Open Game Launcher“) mit der Startoption `--play <id>`. So läuft das Spiel im
// Gaming-Modus wie jedes andere Steam-Spiel (Overlay, Controller-Layout, Fokus).

import { ButtonItem, PanelSection, PanelSectionRow, ToggleField, staticClasses } from "@decky/ui";
import { callable, definePlugin, toaster } from "@decky/api";
import { useEffect, useState } from "react";

type Game = {
  id: string;
  name: string;
  installed: boolean;
  favorite: boolean;
  last_played: number;
  minutes: number;
  source: string;
};
type Launcher = { exe: string; start_dir: string; prefix: string };
type Library = { ok: true; launcher: Launcher; games: Game[] } | { ok: false; reason: string };

const getLibrary = callable<[], Library>("get_library");

const ID_RE = /^[A-Za-z0-9:._/-]{1,200}$/;
const SHORTCUT_NAME = "Open Game Launcher";
const STORE_KEY = "og-decky-shortcut";

declare const SteamClient: any;
declare const appStore: any;

const DE = (navigator.language || "").toLowerCase().startsWith("de");
const T = {
  missing: DE
    ? "Starte den Open Game Launcher einmal im Desktop-Modus (Einstellungen → Allgemein → Verbindungen: „Steam-Deck-Plugin“)."
    : "Start Open Game Launcher once in Desktop Mode (Settings → General → Connections: “Steam Deck plugin”).",
  broken: DE
    ? "Die Spieleliste konnte nicht gelesen werden. Starte den Open Game Launcher einmal, um sie neu zu schreiben."
    : "The game list could not be read. Start Open Game Launcher once to refresh it.",
  reload: DE ? "Neu laden" : "Reload",
  installedOnly: DE ? "Nur installierte" : "Installed only",
  recent: DE ? "Zuletzt gespielt" : "Recently played",
  favorites: DE ? "Favoriten" : "Favorites",
  all: DE ? "Alle Spiele" : "All games",
  less: DE ? "Weniger zeigen" : "Show less",
  notInstalled: DE ? "nicht installiert" : "not installed",
  failed: DE ? "Start fehlgeschlagen" : "Start failed",
};

function fmtMinutes(m: number): string {
  if (m < 60) return `${m} min`;
  return `${Math.round(m / 6) / 10} h`;
}

type Saved = { appId: number; exe: string; prefix: string };

function savedShortcut(l: Launcher): number | null {
  try {
    const v: Saved = JSON.parse(window.localStorage.getItem(STORE_KEY) || "null");
    if (!v || !Number.isInteger(v.appId) || v.appId <= 0 || !appStore?.GetAppOverviewByAppID?.(v.appId)) return null;
    if (v.exe === l.exe && v.prefix === l.prefix) return v.appId;
    // Launcher liegt woanders (z. B. andere Version): alte Verknüpfung entfernen
    SteamClient.Apps.RemoveShortcut(v.appId);
  } catch {
    /* nichts gespeichert */
  }
  return null;
}

async function ensureShortcut(l: Launcher): Promise<number> {
  const existing = savedShortcut(l);
  if (existing) return existing;
  const appId: number = await SteamClient.Apps.AddShortcut(SHORTCUT_NAME, l.exe, l.start_dir, "");
  SteamClient.Apps.SetShortcutName(appId, SHORTCUT_NAME);
  try {
    const v: Saved = { appId, exe: l.exe, prefix: l.prefix };
    window.localStorage.setItem(STORE_KEY, JSON.stringify(v));
  } catch {
    /* ohne Speicher wird beim nächsten Mal neu angelegt */
  }
  return appId;
}

async function play(l: Launcher, g: Game) {
  // Doppelte Prüfung: nur sichere IDs in die Startoptionen
  if (!ID_RE.test(g.id)) return;
  try {
    const appId = await ensureShortcut(l);
    const opts = `${l.prefix ? l.prefix + " " : ""}--play ${g.id}`;
    SteamClient.Apps.SetShortcutLaunchOptions(appId, opts);
    // Kurz warten, bis Steam die Änderung übernommen hat
    await new Promise((r) => setTimeout(r, 300));
    const gameId = ((BigInt(appId) << 32n) | 0x02000000n).toString();
    SteamClient.Apps.RunGame(gameId, "", -1, 100);
  } catch (e) {
    toaster.toast({ title: "Open Game Launcher", body: `${T.failed}: ${String(e)}` });
  }
}

function Content() {
  const [lib, setLib] = useState<Library | null>(null);
  const [all, setAll] = useState(false);
  const [installedOnly, setInstalledOnly] = useState(true);

  const reload = () => {
    setLib(null);
    getLibrary().then(setLib, () => setLib({ ok: false, reason: "broken" }));
  };
  useEffect(reload, []);

  if (!lib) {
    return (
      <PanelSection>
        <PanelSectionRow>…</PanelSectionRow>
      </PanelSection>
    );
  }
  if (!lib.ok) {
    const text = lib.reason === "missing" ? T.missing : T.broken;
    return (
      <PanelSection>
        <PanelSectionRow>
          <div style={{ fontSize: "12px", lineHeight: 1.4 }}>{text}</div>
        </PanelSectionRow>
        <PanelSectionRow>
          <ButtonItem layout="below" onClick={reload}>
            {T.reload}
          </ButtonItem>
        </PanelSectionRow>
      </PanelSection>
    );
  }
  const games = lib.games.filter((g) => !installedOnly || g.installed);
  const recent = games.filter((g) => g.last_played > 0).slice(0, 8);
  const favs = games.filter((g) => g.favorite && !recent.includes(g)).slice(0, 8);
  const rest = all ? games.filter((g) => !recent.includes(g) && !favs.includes(g)) : [];
  const row = (g: Game) => (
    <PanelSectionRow key={g.id}>
      <ButtonItem
        layout="below"
        description={`${g.source}${g.minutes > 0 ? " · " + fmtMinutes(g.minutes) : ""}${g.installed ? "" : " · " + T.notInstalled}`}
        onClick={() => play(lib.launcher, g)}
      >
        {g.name}
      </ButtonItem>
    </PanelSectionRow>
  );
  return (
    <>
      <PanelSection>
        <PanelSectionRow>
          <ToggleField label={T.installedOnly} checked={installedOnly} onChange={setInstalledOnly} />
        </PanelSectionRow>
      </PanelSection>
      {recent.length > 0 && <PanelSection title={T.recent}>{recent.map(row)}</PanelSection>}
      {favs.length > 0 && <PanelSection title={T.favorites}>{favs.map(row)}</PanelSection>}
      <PanelSection title={all ? T.all : ""}>
        {rest.map(row)}
        <PanelSectionRow>
          <ButtonItem layout="below" onClick={() => setAll(!all)}>
            {all ? T.less : `${T.all} (${games.length})`}
          </ButtonItem>
        </PanelSectionRow>
        <PanelSectionRow>
          <ButtonItem layout="below" onClick={reload}>
            {T.reload}
          </ButtonItem>
        </PanelSectionRow>
      </PanelSection>
    </>
  );
}

function Logo() {
  return (
    <svg viewBox="0 0 24 24" width="1em" height="1em" fill="currentColor" aria-hidden="true">
      <rect x="2" y="4" width="20" height="16" rx="4" />
      <text x="12" y="15.5" textAnchor="middle" fontSize="8" fontWeight="900" fill="#000">
        OG
      </text>
    </svg>
  );
}

export default definePlugin(() => ({
  name: "Open Game Launcher",
  titleView: <div className={staticClasses.Title}>Open Game Launcher</div>,
  content: <Content />,
  icon: <Logo />,
  onDismount() {},
}));
