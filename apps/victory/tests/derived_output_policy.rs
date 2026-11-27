use std::process::Command;

#[test]
fn cli_rejects_derived_outputs_inside_the_source_tree() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join(victory_core::policy::ROOT_MARKER), "").unwrap();
    let modes: &[(&[&str], &str)] = &[
        (&["shaders", "--list"], "untranslated.txt"),
        #[cfg(all(windows, feature = "diagnostics"))]
        (&["replay", "missing-capture", "--images"], "images"),
        #[cfg(all(windows, feature = "diagnostics"))]
        (&["replay", "missing-capture", "--retranslate"], "dxil"),
    ];

    for (args, destination) in modes {
        let output = Command::new(env!("CARGO_BIN_EXE_victory"))
            .current_dir(root.path())
            .env("VICTORY_CONFIG", root.path().join("missing.toml"))
            .env_remove("VICTORY_AUDIO_CAPTURE")
            .args(*args)
            .arg(destination)
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(1), "{args:?}: {output:?}");
        assert!(
            stderr.contains("inside the VIctory source tree"),
            "{args:?}: {stderr}"
        );
        assert!(!root.path().join(destination).exists());
    }
}

#[cfg(all(windows, feature = "diagnostics"))]
#[test]
fn retranslate_rejects_a_container_directory_inside_the_source_tree() {
    let root = tempfile::tempdir().unwrap();
    let source_tree = root.path().join("source");
    let capture = source_tree.join("capture");
    std::fs::create_dir_all(&capture).unwrap();
    std::fs::write(source_tree.join(victory_core::policy::ROOT_MARKER), "").unwrap();
    std::fs::write(capture.join("frame.bin"), "").unwrap();
    let destination = root.path().join("dxil");

    let output = Command::new(env!("CARGO_BIN_EXE_victory"))
        .env_remove("VICTORY_AUDIO_CAPTURE")
        .args(["replay", "source/capture", "--retranslate", "dxil"])
        .current_dir(root.path())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(
        stderr.contains("replay shader container directory"),
        "{stderr}"
    );
    assert!(
        stderr.contains("inside the VIctory source tree"),
        "{stderr}"
    );
    assert!(!source_tree.join("containers").exists());
    assert!(!destination.exists());
}
