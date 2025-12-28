use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn go_outputs_info_before_bestmove() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rejectchess"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"uci\ngo\nquit\n")
        .unwrap();

    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    let info_pos = stdout.find("info ").expect("should have info line");
    let bestmove_pos = stdout.find("bestmove ").expect("should have bestmove");
    assert!(
        info_pos < bestmove_pos,
        "info should come before bestmove"
    );
}
