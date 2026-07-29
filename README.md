# Cadence — app desktop (macOS & Windows)

Coquille [Tauri v2](https://tauri.app) de **Cadence**, le centre de contrôle
personnel pour la classe préparatoire. La fenêtre charge l'app web de
production (`https://jarvis-saas-seven.vercel.app`) : l'app est donc toujours
à jour, le binaire ne change presque jamais.

Depuis la **1.2.0**, la coquille se met à jour automatiquement (voir
[§ Mise à jour automatique](#mise-à-jour-automatique)). Les versions ≤ 1.1.0
n'ont pas le client d'update — leurs utilisateurs doivent réinstaller la 1.2.0
**une dernière fois** à la main ; ensuite tout est automatique.

## Téléchargements

- **macOS** (universel, macOS 12+) : [Cadence.dmg](https://github.com/memecoinsfrance-tech/cadence-app/releases/latest/download/Cadence.dmg)
- **Windows** (10/11, x64) : [Cadence-Setup.exe](https://github.com/memecoinsfrance-tech/cadence-app/releases/latest/download/Cadence-Setup.exe)

L'app n'est pas signée par un certificat payant : macOS et Windows demandent
une autorisation au premier lancement — guide sur
[le site](https://cadence-site-gamma.vercel.app#telecharger).

### macOS sans aucun avertissement

L'avertissement Gatekeeper vient de l'attribut `com.apple.quarantine` que le
navigateur pose sur les fichiers téléchargés. `curl` ne le pose pas : cette
commande installe et lance Cadence sans la moindre question.

```bash
curl -fL https://github.com/memecoinsfrance-tech/cadence-app/releases/latest/download/Cadence.dmg -o /tmp/Cadence.dmg && VOL=$(hdiutil attach -nobrowse /tmp/Cadence.dmg | grep -o '/Volumes/.*$' | tail -1) && rm -rf /Applications/Cadence.app && ditto "$VOL/Cadence.app" /Applications/Cadence.app && hdiutil detach "$VOL" -quiet && rm /tmp/Cadence.dmg && open -a Cadence
```

Sur une app déjà installée : `xattr -cr /Applications/Cadence.app`.
Supprimer l'avertissement *à la source* exigerait la notarisation Apple
(programme développeur, 99 €/an) — il n'existe aucun contournement gratuit.

## Développement

```bash
pnpm install
pnpm tauri dev
```

La commande `pnpm tauri dev` ouvre une fenêtre de dev sur l'app de prod.

Build local macOS :

```bash
# `app` est requis dès que bundle.createUpdaterArtifacts est activé (l'artefact
# d'update .app.tar.gz vient du bundle .app, pas du .dmg) ; `--no-sign` évite
# d'avoir à exporter la clé privée pour un simple build de test.
pnpm tauri build --target universal-apple-darwin --bundles app,dmg --no-sign
```

## Mise à jour automatique

La coquille embarque [`tauri-plugin-updater`](https://v2.tauri.app/plugin/updater/).
20 s après le lancement, elle lit `latest.json` sur la dernière release, et si
une version plus récente existe, la télécharge, l'installe et redémarre. Tout est
piloté en **Rust** : aucune permission n'est exposée à la page Vercel distante,
et l'updater ne peut jamais empêcher l'app de démarrer (tâche isolée, erreurs
avalées). Sur macoS l'app se remplace elle-même sans passer par le navigateur,
donc **sans avertissement Gatekeeper** — la mise à jour auto est plus propre que
la réinstallation manuelle.

### Préparer la signature (une seule fois)

Les mises à jour sont signées (minisign). Sans clé, le CI refuse de publier.

```bash
# 1) Générer la paire de clés. Choisis un vrai mot de passe et NOTE-LE :
#    sans lui, aucune mise à jour future ne pourra être signée.
pnpm tauri signer generate -w ~/.tauri/cadence-updater.key

# 2) Coller le contenu de la clé PUBLIQUE dans src-tauri/tauri.conf.json,
#    champ plugins.updater.pubkey (la chaîne entière, PAS un chemin) :
cat ~/.tauri/cadence-updater.key.pub

# 3) Poser les deux secrets GitHub (la clé privée transite par stdin, pas par
#    l'historique shell ; le mot de passe est saisi masqué) :
gh secret set TAURI_SIGNING_PRIVATE_KEY \
  --repo memecoinsfrance-tech/cadence-app < ~/.tauri/cadence-updater.key
gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD \
  --repo memecoinsfrance-tech/cadence-app
```

**Sauvegarde `~/.tauri/cadence-updater.key` + son mot de passe** dans un
gestionnaire de mots de passe. La clé publique correspondante est **compilée
dans chaque binaire installé** : la perdre bloque tout le parc sur sa version
courante, avec une réinstallation manuelle générale pour repartir. Le dépôt
étant public, une fuite permettrait de signer de fausses mises à jour — d'où le
mot de passe, seconde barrière si un secret GitHub fuitait.

### Publier une version

```bash
# Aligner les TROIS fichiers de version sur le tag (le CI le vérifie et refuse
# tout écart) : src-tauri/tauri.conf.json, package.json, src-tauri/Cargo.toml.
git tag v1.2.0
git push origin v1.2.0
```

Le workflow `release.yml` : un job `garde` (versions alignées + secrets
présents) puis les deux builds signés, puis un job `release` qui génère
`latest.json`, publie en brouillon, bascule en « latest » et **vérifie que le
manifeste servi pointe sur des fichiers réellement téléchargeables**. Noms
d'assets stables conservés : `Cadence.dmg` / `Cadence-Setup.exe`.

Deux règles de sûreté, parce que le pire mode de panne (un `latest.json` qui
pointe sur un fichier absent) est silencieux :
- **On ne re-tague jamais.** L'updater compare en semver strict : une release
  re-taguée au même numéro ne sera jamais installée. On incrémente.
- **Le rollback = supprimer la release.** `latest` repointe sur la précédente,
  plus ancienne, donc ignorée par l'updater — ceux qui ne l'ont pas encore
  prise sont protégés.

Avant d'annoncer une version, **tester une vraie mise à jour** (installer la
N-1, publier la N, vérifier qu'elle s'applique) : c'est le seul moyen de
détecter une `pubkey` mal collée, que le CI ne peut pas voir.

## Recette locale

Recette du 2026-07-23, exécutée par un agent **headless** (sans interface, sans
identifiants) juste après le build universel : `pnpm tauri build --target
universal-apple-darwin --bundles app,dmg` → OK en 5 min 38 s →
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
