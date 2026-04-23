use assert_cmd::Command;

#[test]
fn help_flag_exits_zero() {
    let mut cmd = Command::cargo_bin("forge").expect(
        "binary not found: tried 'forge' — check forge_main/Cargo.toml [[bin]] name",
    );
    cmd.arg("--help").assert().success();
}
