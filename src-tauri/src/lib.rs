use tauri::{WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_opener::OpenerExt;

/// La coquille charge l'app web de production — même pattern que l'app iOS
/// Capacitor (`server.url` dans capacitor.config.json de jarvis-saas).
const APP_URL: &str = "https://jarvis-saas-seven.vercel.app/";
const APP_HOST: &str = "jarvis-saas-seven.vercel.app";

/// Injecté avant tout script de page (et à chaque navigation) :
/// 1. marqueur lu par l'app web (jarvis-saas/lib/shell.ts) ;
/// 2. les liens `_blank` / `window.open` sont convertis en navigation de la
///    webview, pour que `on_navigation` (ci-dessous) puisse router l'externe
///    vers le navigateur système — sinon WKWebView/WebView2 les ignorent.
const INIT_SCRIPT: &str = r#"
window.__CADENCE_DESKTOP__ = true;
window.open = function (u) { if (u) window.location.href = String(u); return null; };
document.addEventListener("click", function (e) {
  var t = e.target;
  var a = t && t.closest ? t.closest('a[target="_blank"]') : null;
  if (a && a.href) { e.preventDefault(); window.location.href = a.href; }
}, true);
"#;

/// Hauteur, en points logiques, de la bande réservée en haut de la fenêtre à la
/// barre de titre fondue. 28 pt = la hauteur EXACTE d'une barre de titre macOS
/// standard : les feux tricolores gardent donc leur position native (centrés à
/// y = 14) et on n'a aucune géométrie à deviner. C'est aussi pour ça qu'on
/// n'appelle PAS `traffic_light_position` — son paramètre `y` n'est pas une
/// marge haute mais un redimensionnement de la NSTitlebarContainerView
/// (wry, `inset_traffic_lights`), donc impossible à régler sans lancer l'app.
/// Si un jour on veut un en-tête plus aéré à la Linear (~52 pt), c'est ce
/// chiffre qu'on augmente ET `traffic_light_position` qu'on ajoute, à l'œil.
#[cfg(target_os = "macos")]
const TITLEBAR_H: u32 = 28;

/// Volet macOS de l'init script. Il fait deux choses, et il est essentiel
/// qu'il les fasse ICI plutôt que dans le bundle web :
///
/// 1. `__CADENCE_SHELL__` annonce la hauteur à réserver. Drapeau versionné ET
///    porteur de la valeur : une coquille ≤ 1.0.1 (barre de titre native, déjà
///    installée chez des utilisateurs) ne le pose pas, l'app web retombe donc
///    sur 0 px. Le défaut se trompe toujours du bon côté.
/// 2. Il monte lui-même la zone de glissement. Sous `TitleBarStyle::Overlay`,
///    `titlebarAppearsTransparent` retire la barre native du hit-testing :
///    plus AUCUN glissement natif (tauri-utils : « You need to define a custom
///    drag region »). Si cette zone vivait dans le bundle web, un lancement
///    hors ligne servi par le cache du service worker rendrait la fenêtre
///    impossible à déplacer. Ici elle est indépendante de la version du bundle.
///
/// Le z-index est volontairement extrême : les feux sont des vues natives
/// AU-DESSUS de la WKWebView, ils ne peuvent donc jamais être recouverts.
/// `__TITLEBAR_H__` est substitué depuis `TITLEBAR_H` — une seule source de
/// vérité pour la hauteur, côté Rust.
#[cfg(target_os = "macos")]
const MACOS_TITLEBAR_SCRIPT: &str = r#"
window.__CADENCE_SHELL__ = { v: 2, platform: "macos", titlebar: "overlay", titlebarInset: __TITLEBAR_H__ };
(function () {
  var H = window.__CADENCE_SHELL__.titlebarInset;
  function monter() {
    if (!document.body || document.getElementById("cadence-drag-region")) return;
    var d = document.createElement("div");
    d.id = "cadence-drag-region";
    /* Attribut nu = seuls les clics DIRECTS sur cet élément déclenchent le
       glissement (tauri/src/window/scripts/drag.js). La zone est vide, donc
       tout clic dedans la vise directement. Double-clic = zoom, gratuit. */
    d.setAttribute("data-tauri-drag-region", "");
    d.style.cssText =
      "position:fixed;top:0;left:0;right:0;height:" + H + "px;" +
      "z-index:2147483000;background:transparent;-webkit-user-select:none;user-select:none";
    document.body.appendChild(d);
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", monter, { once: true });
  } else {
    monter();
  }
  /* Appelé depuis Rust (eval) à chaque Resized/Focused. En plein écran natif
     macOS les feux ET la barre disparaissent : garder l'inset y laisserait une
     bande morte permanente — exactement la barre noire qu'on supprime. */
  window.__CADENCE_SET_TITLEBAR__ = function (plein) {
    try {
      var d = document.getElementById("cadence-drag-region");
      if (d) d.style.display = plein ? "none" : "block";
      var r = document.documentElement;
      r.style.setProperty("--titlebar-h", plein ? "0px" : H + "px");
      if (plein) r.removeAttribute("data-titlebar");
      else r.setAttribute("data-titlebar", "overlay");
    } catch (e) {}
  };
})();
"#;

/// Script d'initialisation complet : le tronc commun, plus le volet macOS.
/// Vide hors macOS — Windows garde sa barre de titre système, et l'app web n'y
/// voit pas `__CADENCE_SHELL__`, donc `--titlebar-h` y reste à 0 px.
fn init_script() -> String {
    #[cfg(target_os = "macos")]
    {
        format!(
            "{INIT_SCRIPT}{}",
            MACOS_TITLEBAR_SCRIPT.replace("__TITLEBAR_H__", &TITLEBAR_H.to_string())
        )
    }
    #[cfg(not(target_os = "macos"))]
    {
        INIT_SCRIPT.to_string()
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let url: tauri::Url = APP_URL.parse().expect("APP_URL invalide");
            let builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url))
                .title("Cadence")
                .inner_size(1280.0, 800.0)
                .min_inner_size(980.0, 640.0)
                .initialization_script(init_script())
                .on_navigation(move |url| {
                    let interne = url.host_str() == Some(APP_HOST);
                    if !interne {
                        // Ouvre dans le navigateur par défaut ; la fenêtre
                        // Cadence ne quitte jamais l'app.
                        let _ = handle.opener().open_url(url.as_str(), None::<&str>);
                    }
                    interne
                });

            // Barre de titre fondue (macOS uniquement).
            //
            // Overlay = titlebarAppearsTransparent + fullSizeContentView : la
            // hauteur du contenu devient celle de la fenêtre, la page monte
            // donc jusqu'à y = 0 et les feux tricolores flottent par-dessus.
            // Plus de bande opaque « Cadence » qui casse le dégradé.
            //
            // `hidden_title` est nécessaire EN PLUS : Overlay ne touche pas
            // `titleVisibility`, le titre resterait peint au milieu de l'app.
            //
            // Shadowing plutôt que `let mut` : ces méthodes sont
            // `#[cfg(target_os = "macos")]` sur le builder, donc absentes à la
            // compilation Windows (E0599 sinon). Le shadowing évite en prime le
            // warning `unused_mut` côté Windows — et le job `build-windows` du
            // CI est en `needs:` de la Release : s'il casse, RIEN n'est publié.
            #[cfg(target_os = "macos")]
            let builder = builder
                .title_bar_style(tauri::TitleBarStyle::Overlay)
                .hidden_title(true);

            let fenetre = builder.build()?;

            // Plein écran : aucune API web ne le détecte de façon fiable ici
            // (`display-mode` reflète le mode PWA, `fullscreenElement` l'API
            // DOM, et `innerHeight` est faussé par fullSizeContentView). On
            // pousse donc l'info depuis Rust. `eval` est une injection
            // Rust → JS : elle ne passe PAS par le système de capabilities,
            // contrairement à un `invoke` initié par la page.
            //
            // Toute transition plein écran émet un `Resized` ; on réémet aussi
            // sur `Focused` pour resynchroniser après une navigation, qui
            // rejoue l'init script et remet l'inset à sa valeur par défaut.
            //
            // `is_fullscreen()` est sûr ici : `send_user_message` compare le
            // thread courant au thread principal et exécute en direct quand ils
            // coïncident (tauri-runtime-wry) — or les événements fenêtre sont
            // dispatchés sur le thread principal. Aucun aller-retour de canal,
            // donc aucun interblocage.
            #[cfg(target_os = "macos")]
            {
                let w = fenetre.clone();
                fenetre.on_window_event(move |event| {
                    if matches!(
                        event,
                        tauri::WindowEvent::Resized(_) | tauri::WindowEvent::Focused(_)
                    ) {
                        let plein = w.is_fullscreen().unwrap_or(false);
                        let _ = w.eval(format!(
                            "window.__CADENCE_SET_TITLEBAR__ && window.__CADENCE_SET_TITLEBAR__({plein});"
                        ));
                    }
                });
            }

            // Hors macOS la fenêtre n'est pas réutilisée après construction :
            // sans ça, `unused_variables` casserait un CI en `-D warnings`.
            #[cfg(not(target_os = "macos"))]
            let _ = &fenetre;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("erreur au démarrage de Cadence");
}
