#[cfg(not(feature = "server"))]
use barforge_web::AppEntry;
#[cfg(feature = "server")]
use barforge_web::server;

fn main() {
    #[cfg(feature = "server")]
    dioxus::serve(|| async move {
        let env = |key: &str| std::env::var(key).ok();
        let _ = server::init_tracing();
        let auth_state = server::auth_state_from_env(&env)?;
        let session_layer = server::session_layer_from_env(&env).await?;
        Ok(server::app_router(auth_state, session_layer))
    });

    #[cfg(not(feature = "server"))]
    dioxus::LaunchBuilder::new()
        .with_cfg(dioxus::web::Config::new().hydrate(true).rootname("body"))
        .launch(AppEntry);
}
