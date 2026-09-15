fn main() {
    // The app's own commands get permissions only when listed here; the
    // capability file for the update window refers to them by name.
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(&["install_update", "snooze_update", "toggle_fullscreen"])),
    )
    .expect("tauri-build");
}
