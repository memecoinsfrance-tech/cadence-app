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
