use std::fs;

use duck_core::{AppPaths, Context, Sysroot};

use crate::{
    model::{Backend, SaveRequest, TargetEntry, TargetMode},
    save, status,
};

struct Fixture {
    ctx: Context,
    _root: std::path::PathBuf,
}

fn fixture(name: &str) -> Fixture {
    let root = std::env::temp_dir().join(format!("duck-ts-e2e-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let module = root.join("module");
    let sys = root.join("sys");
    fs::create_dir_all(module.join("bin")).unwrap();
    fs::write(module.join("module.prop"), "id=duck-toolbox\n").unwrap();
    let paths = AppPaths::for_root(&module, &root.join("data"));
    Fixture {
        ctx: Context {
            paths,
            sysroot: Sysroot::new(&sys),
        },
        _root: root,
    }
}

fn install_backend(sysroot: &Sysroot, id: &str, version_code: u64) {
    let dir = sysroot.path(format!("/data/adb/modules/{id}"));
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("module.prop"),
        format!("id={id}\nname={id}\nversionCode={version_code}\n"),
    )
    .unwrap();
}

fn config_path(fixture: &Fixture, device_path: &str) -> std::path::PathBuf {
    fixture.ctx.sysroot.path(device_path)
}

#[test]
fn status_reports_no_backend_when_none_installed() {
    let fixture = fixture("none");
    let status = status(&fixture.ctx).unwrap();

    assert!(status.active.is_none());
    assert!(status.backends.is_empty());
    assert!(status.schema.is_none());
    // Default system apps are still offered so the picker is populated.
    assert!(!status.system_apps.is_empty());
}

#[test]
fn tricky_store_ini_round_trips_targets_and_modes() {
    let fixture = fixture("ts-ini");
    install_backend(&fixture.ctx.sysroot, "tricky_store", 246);

    let data = save(
        &fixture.ctx,
        SaveRequest {
            targets: vec![
                TargetEntry {
                    package_name: "com.google.android.gms".into(),
                    mode: TargetMode::Generate,
                },
                TargetEntry {
                    package_name: "io.github.vvb2060.keyattestation".into(),
                    mode: TargetMode::Auto,
                },
            ],
            default_policy: [("os_patch".to_owned(), "prop".to_owned())].into(),
            per_app_policy: [(
                "com.google.android.gms".to_owned(),
                [("boot_patch".to_owned(), "20260101".to_owned())].into(),
            )]
            .into(),
            system_apps: vec!["com.google.android.gms".into()],
            auto_add_new_apps: false,
        },
    )
    .unwrap();

    assert_eq!(data.backend, Backend::TrickyStore);
    assert_eq!(data.target_count, 2);

    let written =
        fs::read_to_string(config_path(&fixture, "/data/adb/tricky_store/config.ini")).unwrap();
    assert!(written.contains("com.google.android.gms!"));
    assert!(written.contains("[default_policy]"));
    assert!(written.contains("[com.google.android.gms]"));

    let status = status(&fixture.ctx).unwrap();
    assert_eq!(status.active.unwrap().backend, Backend::TrickyStore);
    assert!(status.schema.unwrap().supports_per_app_policy);
    assert_eq!(status.config.targets.len(), 2);
    assert!(status.keybox.is_some());
}

#[test]
fn legacy_tricky_store_uses_target_txt() {
    let fixture = fixture("ts-legacy");
    install_backend(&fixture.ctx.sysroot, "tricky_store", 200);

    save(
        &fixture.ctx,
        SaveRequest {
            targets: vec![TargetEntry {
                package_name: "com.a".into(),
                mode: TargetMode::Hack,
            }],
            default_policy: [("os_patch".to_owned(), "no".to_owned())].into(),
            ..SaveRequest::default()
        },
    )
    .unwrap();

    let target =
        fs::read_to_string(config_path(&fixture, "/data/adb/tricky_store/target.txt")).unwrap();
    assert_eq!(target.trim(), "com.a?");
    // A no-op security patch removes the file rather than writing an empty policy.
    assert!(!config_path(&fixture, "/data/adb/tricky_store/security_patch.txt").exists());
}

#[test]
fn tee_simulator_writes_profiles_json() {
    let fixture = fixture("tees");
    install_backend(&fixture.ctx.sysroot, "teesim", 400);
    let keybox = config_path(&fixture, "/data/adb/teesim/keybox.xml");
    fs::create_dir_all(keybox.parent().unwrap()).unwrap();
    fs::write(&keybox, crate::keybox::AOSP_KEYBOX).unwrap();

    save(
        &fixture.ctx,
        SaveRequest {
            targets: vec![TargetEntry {
                package_name: "com.google.android.gms".into(),
                // Generate mode is not supported and must be flattened to auto.
                mode: TargetMode::Generate,
            }],
            default_policy: [("os_patch".to_owned(), "today".to_owned())].into(),
            ..SaveRequest::default()
        },
    )
    .unwrap();

    let raw = fs::read_to_string(config_path(&fixture, "/data/adb/teesim/config.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(
        json["profiles"]["default"]["apps"][0],
        "com.google.android.gms"
    );
    assert_eq!(json["profiles"]["default"]["patchLevel"]["system"], "today");

    let status = status(&fixture.ctx).unwrap();
    assert_eq!(status.config.targets[0].mode, TargetMode::Auto);
    assert!(!status.schema.unwrap().supports_app_mode);
}

#[test]
fn oh_my_keymint_splits_scoop_and_trust() {
    let fixture = fixture("omk");
    install_backend(&fixture.ctx.sysroot, "oh_my_keymint", 100);
    let seeded = config_path(&fixture, "/data/misc/keystore/omk/config.toml");
    fs::create_dir_all(seeded.parent().unwrap()).unwrap();
    fs::write(
        &seeded,
        "version = 2\n\n[trust]\nos_version = \"auto\"\nsecurity_patch = \"latest\"\nverified_boot_state = true\ndevice_locked = true\n",
    )
    .unwrap();

    save(
        &fixture.ctx,
        SaveRequest {
            targets: vec![TargetEntry {
                package_name: "com.android.vending".into(),
                mode: TargetMode::Auto,
            }],
            default_policy: [
                ("os_version".to_owned(), "15".to_owned()),
                ("security_patch".to_owned(), "auto".to_owned()),
            ]
            .into(),
            ..SaveRequest::default()
        },
    )
    .unwrap();

    let injector = fs::read_to_string(config_path(
        &fixture,
        "/data/misc/keystore/omk/injector.toml",
    ))
    .unwrap();
    assert!(injector.contains("com.android.vending"));
    let config =
        fs::read_to_string(config_path(&fixture, "/data/misc/keystore/omk/config.toml")).unwrap();
    assert!(config.contains("os_version = 15"));
    assert!(config.contains("security_patch = \"auto\""));
}

#[test]
fn save_rejects_invalid_policy_value() {
    let fixture = fixture("bad-policy");
    install_backend(&fixture.ctx.sysroot, "tricky_store", 246);

    let error = save(
        &fixture.ctx,
        SaveRequest {
            targets: vec![TargetEntry {
                package_name: "com.a".into(),
                mode: TargetMode::Auto,
            }],
            default_policy: [("os_patch".to_owned(), "not-a-date".to_owned())].into(),
            ..SaveRequest::default()
        },
    )
    .unwrap_err();

    assert!(error.to_string().contains("os_patch"));
}
