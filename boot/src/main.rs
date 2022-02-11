use fern::colors::{Color, ColoredLevelConfig};
use log::{debug, info};
use std::{
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
    time::Duration,
};

const TEST_TIMEOUT_SECS: u64 = 30;

fn main() {
    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Green)
        .debug(Color::Magenta);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                chrono::Local::now().format("[%H:%M:%S]"),
                colors.color(record.level()),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    let mut args = std::env::args().skip(1); // skip executable name

    let kernel_binary_path = PathBuf::from(args.next().unwrap()).canonicalize().unwrap();

    let no_boot = if let Some(arg) = args.next() {
        match arg.as_str() {
            "--no-run" => true,
            other => panic!("unexpected argument `{}`", other),
        }
    } else {
        false
    };

    let image = create_disk_images(&kernel_binary_path);

    if no_boot {
        info!("created disk image at `{}`", image.display());
        return;
    }

    info!("booting {}", image.display());

    let mut run_cmd = Command::new("qemu-system-x86_64");
    run_cmd
        .arg("-drive")
        .arg(format!("format=raw,file={}", image.display()));

    let binary_kind = runner_utils::binary_kind(&kernel_binary_path);
    if binary_kind.is_test() {
        run_cmd.args(test_args());
        debug!("{:?}\n", run_cmd);

        let exit_status = run_test_command(run_cmd);
        match exit_status.code() {
            Some(33) => {} // success
            Some(other) => panic!("Test failed (exit code: {:?})", other),
            None => panic!("Test failed (no exit code)"),
        }
    } else {
        run_cmd.args(run_args());
        debug!("{:?}\n", run_cmd);

        let exit_status = run_cmd.status().unwrap();
        if !exit_status.success() {
            std::process::exit(exit_status.code().unwrap_or(1));
        }
    }
}

fn run_args() -> Vec<&'static str> {
    let mut vec = base_args();
    vec.push("-s");
    vec.push("-monitor");
    vec.push("telnet::45454,server,nowait");
    vec
}

fn test_args() -> Vec<&'static str> {
    let mut vec = base_args();
    vec.push("-display");
    vec.push("none");
    vec.push("-device");
    vec.push("isa-debug-exit,iobase=0xf4,iosize=0x04");
    vec
}

fn base_args() -> Vec<&'static str> {
    let mut vec = Vec::new();
    vec.push("--no-reboot");
    vec.push("-serial");
    vec.push("stdio");
    vec.push("-drive");
    vec.push("file=disk.img,if=ide,format=raw");
    vec
}

fn run_test_command(mut cmd: Command) -> ExitStatus {
    runner_utils::run_with_timeout(&mut cmd, Duration::from_secs(TEST_TIMEOUT_SECS)).unwrap()
}

pub fn create_disk_images(kernel_binary_path: &Path) -> PathBuf {
    info!("creating disk images in {}", kernel_binary_path.display());

    let bootloader_manifest_path = bootloader_locator::locate_bootloader("bootloader").unwrap();
    let kernel_manifest_path = locate_cargo_manifest::locate_manifest().unwrap();

    let mut build_cmd = Command::new(env!("CARGO"));
    build_cmd.current_dir(bootloader_manifest_path.parent().unwrap());
    build_cmd.arg("builder");
    build_cmd
        .arg("--kernel-manifest")
        .arg(&kernel_manifest_path);
    build_cmd.arg("--kernel-binary").arg(&kernel_binary_path);
    build_cmd
        .arg("--target-dir")
        .arg(kernel_manifest_path.parent().unwrap().join("target"));
    build_cmd
        .arg("--out-dir")
        .arg(kernel_binary_path.parent().unwrap());
    build_cmd.arg("--quiet");

    debug!("{:?}", build_cmd);

    if !build_cmd.status().unwrap().success() {
        panic!("build failed");
    }

    let kernel_binary_name = kernel_binary_path.file_name().unwrap().to_str().unwrap();
    let disk_image = kernel_binary_path
        .parent()
        .unwrap()
        .join(format!("boot-bios-{}.img", kernel_binary_name));
    if !disk_image.exists() {
        panic!(
            "Disk image does not exist at {} after bootloader build",
            disk_image.display()
        );
    }
    disk_image
}
