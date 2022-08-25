//! `cargo r` will run this testing binary in debug mode but run all examples in debug mode
//! `RELEASE=1 cargo r` will run this testing binary in debug mode but run all examples in release mode

use expect_test::{expect, Expect};

fn main() {
    // println!("testing");
    // let [out, err] = run("./mechanic/linux.sh", &[]).expect("mechanic failed");
    // println!("{out}\n{err}");
    expect!["2"].assert_eq("1");
    // check(&out, expect![[""]]);
    // expect![[""]].assert_eq(&err);
}

fn check(actual: &str, expect: Expect) {
    let actual = actual.to_string();
    expect.assert_eq(&actual);
}

fn run(p: &str, envs: &[(&str, &str)]) -> std::io::Result<[String; 2]> {
    let path: &std::path::Path = p.as_ref();
    let mut cmd = std::process::Command::new("/bin/bash");

    // Clear env if `envs` are specified.
    if !envs.is_empty() {
        cmd.env_clear().envs(envs.iter().copied());
    }

    cmd.current_dir(path.parent().unwrap())
        .arg(path.file_name().unwrap())
        .output()
        .map(|o| {
            [
                String::from_utf8(o.stdout).unwrap(),
                String::from_utf8(o.stderr).unwrap(),
            ]
        })
}
