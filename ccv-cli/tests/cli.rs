use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("ccv-cli")?;

    cmd.arg("validate").arg("test/file/doesnt/exist");
    cmd.assert().failure().stderr(predicate::str::contains(
        r#"Error reading "test/file/doesnt/exist": No such file or directory (os error 2)"#,
    ));

    Ok(())
}

#[test]
fn valid_network_config() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("sample.txt")?;
    file.write_str(
        r#"
network:
  version: 1
  config:
    - type: bond
      name: a
      mac_address: aa:bb
      mtu: 1
      subnets:
        - type: dhcp6
          control: manual
          netmask: 255.255.255.0
          gateway: 10.0.0.1
          dns_nameservers:
            - 8.8.8.8
          dns_search:
            - find.me
          routes:
            - type: route
              destination: 10.20.0.0/8
              gateway: a.b.c.d
              metric: 200"#,
    )?;

    let mut cmd = Command::cargo_bin("ccv-cli")?;
    cmd.arg("validate")
        .args(["--kind", "networkconfig"])
        .arg(file.path());
    cmd.assert().success().stdout(predicate::str::contains(
        r#"{"annotations":[],"errors":[],"is_valid":true}"#,
    ));

    Ok(())
}