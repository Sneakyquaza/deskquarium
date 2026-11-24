//! Windows : manifeste d'application embarqué dans l'exe (systèmes pris en charge pour les
//! fenêtres enfants WS_EX_LAYERED, DPI par écran), compilé par windres de mingw.

fn main() {
    println!("cargo:rerun-if-changed=ressources/aquarium.manifest");
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let sortie = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let manifeste = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("ressources/aquarium.manifest")
        .display()
        .to_string()
        .replace('\\', "/");
    let rc = sortie.join("aquarium.rc");
    // 1 = CREATEPROCESS_MANIFEST_RESOURCE_ID, 24 = RT_MANIFEST.
    std::fs::write(&rc, format!("1 24 \"{manifeste}\"\n")).expect("aquarium.rc");
    let obj = sortie.join("aquarium_ressources.o");
    let windres = std::env::var("WINDRES").unwrap_or_else(|_| {
        if cfg!(windows) {
            "windres".into()
        } else {
            "x86_64-w64-mingw32-windres".into()
        }
    });
    let ok = std::process::Command::new(&windres)
        .arg(&rc)
        .args(["-O", "coff", "-o"])
        .arg(&obj)
        .status()
        .unwrap_or_else(|e| panic!("{windres} introuvable ({e}) : WINDRES pour le chemin"));
    assert!(ok.success(), "{windres} a échoué");
    println!("cargo:rustc-link-arg-bins={}", obj.display());
}
