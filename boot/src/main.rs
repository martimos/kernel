use std::{
    path::{Path, PathBuf},
    process::Command,
};

use clap::Parser;
use fern::colors::{Color, ColoredLevelConfig};
use log::{debug, info};

const TEST_TIMEOUT_SECS: u64 = 30;

mod test;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(parse(from_os_str))]
    binary: PathBuf,
    #[clap(short, long, help = "Print debug information")]
    verbose: bool,
    #[clap(long, help = "Only create the bootable image, don't run it")]
    no_run: bool,
}

fn main() {
    let args: Args = Args::parse();

    configure_logging(args.verbose);

    let kernel_binary_path = args.binary.canonicalize().unwrap();
    let image = create_disk_images(&kernel_binary_path);

    if args.no_run {
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
        let file_name = kernel_binary_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
            .rsplit_once("-")
            .expect("should be of form <image>-<random>")
            .0
            .to_string();
        test::run_test_binary(file_name, run_cmd)
    } else {
        run_cmd.args(vec![
            "--no-reboot",
            "-serial",
            "stdio",
            "-s", // -gdb tcp::1234
            "-monitor",
            "telnet::45454,server,nowait",
        ]);
        debug!("{:?}\n", run_cmd);

        let exit_status = run_cmd.status().unwrap();
        if !exit_status.success() {
            std::process::exit(exit_status.code().unwrap_or(1));
        }
    }
}

fn configure_logging(verbose: bool) {
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
        .level(if verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .chain(std::io::stdout())
        .apply()
        .unwrap();
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

    // debug!("{:?}", build_cmd);

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
