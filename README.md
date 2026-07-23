# Cadence — app desktop (macOS & Windows)

Coquille [Tauri v2](https://tauri.app) de **Cadence**, le centre de contrôle
personnel pour la classe préparatoire. La fenêtre charge l'app web de
production (`https://jarvis-saas-seven.vercel.app`) : l'app est donc toujours
à jour, le binaire ne change presque jamais.

## Téléchargements

- **macOS** (universel, macOS 12+) : [Cadence.dmg](https://github.com/memecoinsfrance-tech/cadence-app/releases/latest/download/Cadence.dmg)
- **Windows** (10/11, x64) : [Cadence-Setup.exe](https://github.com/memecoinsfrance-tech/cadence-app/releases/latest/download/Cadence-Setup.exe)

L'app n'est pas signée par un certificat payant : macOS et Windows demandent
une autorisation au premier lancement (voir le guide sur le site).

## Développement

```bash
pnpm install
pnpm tauri dev
```

La commande `pnpm tauri dev` ouvre une fenêtre de dev sur l'app de prod.

Build local macOS :

```bash
pnpm tauri build --target universal-apple-darwin --bundles dmg
```

## Publier une version

```bash
git tag v1.0.1
git push origin v1.0.1
```

Le workflow `release.yml` construit les deux plateformes et publie la
Release avec les noms d'assets stables `Cadence.dmg` / `Cadence-Setup.exe`.

## Recette locale

Recette du 2026-07-23, exécutée par un agent **headless** (sans interface, sans
identifiants) juste après le build universel : `pnpm tauri build --target
universal-apple-darwin --bundles dmg` → OK en 5 min 38 s →
`Cadence_1.0.0_universal.dmg` (5,5 Mo — plus léger que prévu, logique : sans
frontend embarqué, le dmg ne contient que le binaire universel `x86_64 arm64`
(confirmé par `lipo -info`) et les icônes). Installation headless (`hdiutil
attach` → `ditto` vers `/Applications` → `hdiutil detach`) : OK, sans
attribut `com.apple.quarantine` (attendu pour un build local, cf. note
Gatekeeper ci-dessous). Lancement (`open -a Cadence`) : process stable ≥ 20 s,
`log show` confirme que la WebView charge réellement l'app de prod (audio
WebKit, navigation) sans crash ni entrée dans
`~/Library/Logs/DiagnosticReports` ; quitté proprement. App laissée installée
dans `/Applications` pour la recette visuelle.

Ce qu'un agent headless ne peut pas vérifier — **différé à la recette finale
(Tâche 10, avec Maxence)**, jamais tenté ici (aucune saisie d'identifiants) :
- ☐ Rendu visuel de la fenêtre et icône dans le Dock
- ☐ Connexion email + mot de passe, puis persistance après ⌘Q/relance
- ☐ Fluidité de la navigation (Aujourd'hui, Calendrier, Flashcards)
- ☐ Lien externe → ouverture dans le navigateur par défaut
- ☐ Flux Gatekeeper (nécessite un vrai téléchargement depuis le site, pas un build local)

**Verdict hors-ligne : NON TESTÉ → variante conservatrice (OFFLINE_KO)
retenue pour la FAQ du site.** Le Wi-Fi de la machine de build n'a pas été
coupé (poste en usage actif) ; à re-tester en Tâche 10 avant de trancher
définitivement.
