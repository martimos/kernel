use std::process::Stdio;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

use clap::Parser;
use fern::colors::{Color, ColoredLevelConfig};
use log::{debug, info};

const TEST_TIMEOUT_SECS: u64 = 30;
const QEMU_COMMAND: &str = "qemu-system-x86_64";

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
    #[clap(long, help = "Run Qemu in fullscreen.", conflicts_with = "no-run")]
    fullscreen: bool,
    #[clap(long, help = "Don't use a Qemu accelerator")]
    no_accel: bool,
    #[clap(
        long,
        help = "Start a gdb server on tcp:1234 and wait until a client has connected"
    )]
    debug: bool,
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

    let mut run_cmd = Command::new(QEMU_COMMAND);
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
            .rsplit_once('-')
            .expect("should be of form <image>-<random>")
            .0
            .to_string();
        test::run_test_binary(file_name, run_cmd)
    } else {
        let mut run_args = vec![
            "--no-reboot",
            "-serial",
            "stdio",
            "-monitor",
            "telnet::45454,server,nowait",
            "-drive",
            "file=disk/test_ext2.img,if=ide,format=raw",
        ];
        if args.fullscreen {
            run_args.push("-full-screen");
        }
        if args.debug {
            run_args.push("-s"); // -gdb tcp:1234
            run_args.push("-S");
        }
        if args.no_accel {
            debug!("not using an accelerator");
        } else {
            // use an accelerator
            let available_accels = get_available_accels();
            debug!(
                "available accelerators on this system: {:?}",
                available_accels
            );
            if available_accels.contains(&"kvm".to_string()) {
                debug!("using KVM as accelerator");
                run_args.push("-enable-kvm");
            } else if available_accels.contains(&"hvf".to_string()) {
                debug!("using HVF as accelerator");
                run_args.push("-accel");
                run_args.push("hvf");
            } else {
                debug!("neither HVF nor KVM available, not using an accelerator");
            }
        }
        run_cmd.args(run_args);
        debug!("{:?}\n", run_cmd);

        let exit_status = run_cmd.status().unwrap();
        if !exit_status.success() {
            std::process::exit(exit_status.code().unwrap_or(1));
        }
    }
}

fn get_available_accels() -> Vec<String> {
    let mut cmd = Command::new(QEMU_COMMAND);
    cmd.stdout(Stdio::piped());
    cmd.arg("-accel").arg("help");
    let exit_status = cmd.status().unwrap();
    if !exit_status.success() {
        debug!(
            "unable to list qemu accelerators: exit code {}",
            exit_status
        );
        return Vec::new();
    }

    let mut accelerators = Vec::new();
    let output = cmd.output().unwrap();
    let output_string = String::from_utf8_lossy(&output.stdout);
    let output_lines = output_string.lines();
    output_lines
        .skip(1) // skip the first line, accelerators start from the second line
        .for_each(|line| accelerators.push(line.to_string()));
    accelerators
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
