//! Windows : manifeste d'application embarqué dans l'exe (systèmes pris en charge pour les
//! fenêtres enfants WS_EX_LAYERED, DPI par écran) et identité du programme (VERSIONINFO : nom
//! montré par le Gestionnaire des tâches, les propriétés du fichier), compilés par windres.

fn main() {
    println!("cargo:rerun-if-changed=ressources/aquarium.manifest");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let sortie = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let manifeste = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("ressources/aquarium.manifest")
        .display()
        .to_string()
        .replace('\\', "/");
    let rc = sortie.join("deskquarium.rc");
    let version = env!("CARGO_PKG_VERSION");
    let mut n: Vec<&str> = version.split('.').collect();
    n.resize(4, "0");
    let num = n.join(",");
    // 1 = CREATEPROCESS_MANIFEST_RESOURCE_ID, 24 = RT_MANIFEST.
    let texte = format!(
        "1 24 \"{manifeste}\"\n\
         1 VERSIONINFO\nFILEVERSION {num}\nPRODUCTVERSION {num}\nBEGIN\n\
         BLOCK \"StringFileInfo\"\nBEGIN\nBLOCK \"040904b0\"\nBEGIN\n\
         VALUE \"CompanyName\", \"Sneakyquaza\"\n\
         VALUE \"ProductName\", \"Deskquarium\"\n\
         VALUE \"FileDescription\", \"Deskquarium\"\n\
         VALUE \"InternalName\", \"deskquarium\"\n\
         VALUE \"OriginalFilename\", \"deskquarium.exe\"\n\
         VALUE \"FileVersion\", \"{version}\"\n\
         VALUE \"ProductVersion\", \"{version}\"\n\
         END\nEND\nBLOCK \"VarFileInfo\"\nBEGIN\nVALUE \"Translation\", 0x409, 1200\nEND\nEND\n"
    );
    std::fs::write(&rc, texte).expect("deskquarium.rc");
    let obj = sortie.join("deskquarium_ressources.o");
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
