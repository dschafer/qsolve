use assert_cmd::prelude::*;
#[cfg(feature = "image")]
use std::fs;
use std::process::Command;

#[test]
fn print_succeeds_on_text() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("qsolve")?;

    cmd.arg("print").arg("games/linkedin-1-empty.txt");
    cmd.assert().success();

    Ok(())
}

#[test]
fn print_succeeds_when_clearing() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("qsolve")?;

    cmd.arg("print")
        .arg("games/linkedin-1-partial.txt")
        .arg("--clear");
    cmd.assert().success();

    Ok(())
}

#[cfg(feature = "image")]
#[test]
fn print_succeeds_on_image() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("qsolve")?;

    cmd.arg("print").arg("games/linkedin-1.png");
    cmd.assert().success();

    Ok(())
}

#[test]
fn print_fails_on_bad_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("qsolve")?;

    cmd.arg("print").arg("games/bad-file-does-not-exist.txt");
    cmd.assert().failure();

    Ok(())
}

#[cfg(feature = "image")]
#[test]
fn auto_uses_extension_to_select_parser() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::temp_dir().join(format!(
        "qsolve-text-with-image-extension-{}.png",
        std::process::id()
    ));
    fs::write(&path, "wwww\nwkkk\nrrrr\nbbbb")?;

    let mut cmd = Command::cargo_bin("qsolve")?;
    cmd.arg("print").arg(&path);
    cmd.assert().failure();

    fs::remove_file(path)?;
    Ok(())
}

#[cfg(feature = "image")]
#[test]
fn print_accepts_image_file_type() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("qsolve")?;

    cmd.arg("print")
        .arg("games/linkedin-1.png")
        .arg("--file-type=image");
    cmd.assert().success();

    Ok(())
}

#[test]
fn print_accepts_text_file_type() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("qsolve")?;

    cmd.arg("print")
        .arg("games/linkedin-1-empty.txt")
        .arg("--file-type=text");
    cmd.assert().success();

    Ok(())
}

#[cfg(feature = "image")]
#[test]
fn print_fails_on_bad_file_type() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("qsolve")?;

    cmd.arg("print")
        .arg("games/linkedin-1-empty.txt")
        .arg("--file-type=image"); // This is backward by design!
    cmd.assert().failure();

    Ok(())
}

#[cfg(not(feature = "image"))]
#[test]
fn print_rejects_image_file_type_without_image_feature() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("qsolve")?;

    cmd.arg("print")
        .arg("games/linkedin-1.png")
        .arg("--file-type=image");
    cmd.assert().failure();

    Ok(())
}

#[test]
fn animate_succeeds_on_text() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("qsolve")?;

    cmd.arg("animate")
        .arg("games/linkedin-1-empty.txt")
        .arg("--delay=1");
    cmd.assert().success();

    Ok(())
}

#[test]
fn solve_succeeds_on_text() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("qsolve")?;

    cmd.arg("solve").arg("games/linkedin-1-empty.txt");
    cmd.assert().success();

    Ok(())
}

#[test]
fn solve_succeeds_with_share() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("qsolve")?;

    cmd.arg("solve")
        .arg("games/linkedin-1-empty.txt")
        .arg("--share");
    cmd.assert().success();

    Ok(())
}

#[test]
fn solve_succeeds_with_share_text() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("qsolve")?;

    cmd.arg("solve")
        .arg("games/linkedin-1-empty.txt")
        .arg("--share=\"123\"");
    cmd.assert().success();

    Ok(())
}

#[test]
fn profile_succeeds_on_text() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("qsolve")?;

    cmd.arg("profile").arg("games/linkedin-1-empty.txt");
    cmd.assert().success();

    Ok(())
}

#[test]
fn hint_succeeds_on_text() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("qsolve")?;

    cmd.arg("hint").arg("games/linkedin-1-empty.txt");
    cmd.assert().success();

    Ok(())
}
