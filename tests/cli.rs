//! Command line argument tests
use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn deposits() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("deposits.csv")?;
    file.write_str(
        r#"type,client,tx,amount
deposit,1,1,1.0
deposit,1,3,2.0
"#,
    )?;
    let mut cmd = Command::cargo_bin("transactions")?;

    cmd.arg(file.path());
    cmd.assert().stdout(predicate::eq(
        br#"client,available,held,total,locked
1,3.0,0.0,3.0,false
"# as &[u8],
    ));

    Ok(())
}

// Test that the deposits with 4 decimal places add up correctly
#[test]
fn deposits_total() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("deposits2.csv")?;
    let mut test_data = "type,client,tx,amount\n".to_string();
    for tx in 1usize..10000 {
        test_data.push_str(&format!("deposit,1,{},0.0001\n", tx));
    }
    file.write_str(&test_data)?;
    let mut cmd = Command::cargo_bin("transactions")?;
    std::fs::copy(file.path(), "/tmp/deposits2.csv")?;

    cmd.arg(file.path());
    cmd.assert().stdout(predicate::eq(
        br#"client,available,held,total,locked
1,0.9999,0.0,0.9999,false
"# as &[u8],
    ));

    Ok(())
}

#[test]
fn withdrawls() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("withdrawls.csv")?;
    file.write_str(
        r#"type,client,tx,amount
deposit,1,1,1.0
deposit,1,3,2.0
withdrawal,1,4,1.5
"#,
    )?;
    let mut cmd = Command::cargo_bin("transactions")?;

    cmd.arg(file.path());
    cmd.assert().stdout(predicate::eq(
        br#"client,available,held,total,locked
1,1.5,0.0,1.5,false
"# as &[u8],
    ));

    Ok(())
}

#[test]
fn withdrawls_overdraft() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("withdrawls_overdraft.csv")?;
    file.write_str(
        r#"type,client,tx,amount
deposit,1,1,1.0
withdrawal,1,2,5.0
"#,
    )?;
    let mut cmd = Command::cargo_bin("transactions")?;

    cmd.arg(file.path());
    cmd.assert().stdout(predicate::eq(
        br#"client,available,held,total,locked
1,1.0,0.0,1.0,false
"# as &[u8],
    ));

    Ok(())
}

#[test]
fn dispute() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("dispute.csv")?;
    file.write_str(
        r#"type,client,tx,amount
deposit,1,1,1.0
dispute,1,1,
"#,
    )?;
    let mut cmd = Command::cargo_bin("transactions")?;

    cmd.arg(file.path());
    cmd.assert().stdout(predicate::eq(
        br#"client,available,held,total,locked
1,0.0,1.0,1.0,false
"# as &[u8],
    ));

    Ok(())
}

#[test]
fn resolve() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("resolve.csv")?;
    file.write_str(
        r#"type,client,tx,amount
deposit,1,1,1.0
dispute,1,1,
resolve,1,1,
"#,
    )?;
    let mut cmd = Command::cargo_bin("transactions")?;

    cmd.arg(file.path());
    cmd.assert().stdout(predicate::eq(
        br#"client,available,held,total,locked
1,1.0,0.0,1.0,false
"# as &[u8],
    ));

    Ok(())
}

#[test]
fn chargeback() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("chargeback.csv")?;
    file.write_str(
        r#"type,client,tx,amount
deposit,1,1,1.0
dispute,1,1,
chargeback,1,1,
"#,
    )?;
    let mut cmd = Command::cargo_bin("transactions")?;

    cmd.arg(file.path());
    cmd.assert().stdout(predicate::eq(
        br#"client,available,held,total,locked
1,0.0,0.0,0.0,true
"# as &[u8],
    ));

    Ok(())
}