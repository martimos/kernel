use std::collections::BTreeMap;
use std::fs;
use std::process::{Command, ExitStatus};
use std::time::Duration;

use log::{debug, info};

use crate::TEST_TIMEOUT_SECS;

#[derive(Debug, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct QemuConfig {
    all_tests: Vec<String>,
    tests: BTreeMap<String, Vec<String>>,
}

pub fn run_test_binary(test_name: String, mut run_cmd: Command) {
    let data = fs::read("tests/qemu_config.yaml").unwrap();
    let config_content = String::from_utf8_lossy(&data);
    let config: QemuConfig = serde_yaml::from_str(&config_content).unwrap();

    let mut args: Vec<String> = config
        .all_tests
        .iter()
        .flat_map(|s| s.split(' '))
        .map(|s| s.to_string())
        .collect();
    if let Some(additional_args) = config.tests.get(&test_name) {
        info!("found additional qemu arguments for test '{}'", test_name);
        additional_args
            .iter()
            .flat_map(|s| s.split(' '))
            .for_each(|e| args.push(e.to_string()))
    }

    run_cmd.args(args);
    debug!("{:?}\n", run_cmd);

    let exit_status = run_test_command(run_cmd);
    match exit_status.code() {
        Some(33) => {} // success
        Some(other) => panic!("Test failed (exit code: {:?})", other),
        None => panic!("Test failed (no exit code)"),
    }
}

fn run_test_command(mut cmd: Command) -> ExitStatus {
    runner_utils::run_with_timeout(&mut cmd, Duration::from_secs(TEST_TIMEOUT_SECS)).unwrap()
}
