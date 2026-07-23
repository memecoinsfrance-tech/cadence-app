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

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let url: tauri::Url = APP_URL.parse().expect("APP_URL invalide");
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url))
                .title("Cadence")
                .inner_size(1280.0, 800.0)
                .min_inner_size(980.0, 640.0)
                .initialization_script(INIT_SCRIPT)
                .on_navigation(move |url| {
                    let interne = url.host_str() == Some(APP_HOST);
                    if !interne {
                        // Ouvre dans le navigateur par défaut ; la fenêtre
                        // Cadence ne quitte jamais l'app.
                        let _ = handle.opener().open_url(url.as_str(), None::<&str>);
                    }
                    interne
                })
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("erreur au démarrage de Cadence");
}
