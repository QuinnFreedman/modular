import subprocess
import os
import os.path as path
from enum import Enum
import tempfile
import shutil
import sys
import re
from typing import List, Dict, Optional

class GitDiff(Enum):
    NO_CHANGE = 0
    NO_LAST_COMMIT = 1
    CHANGE_SINCE_LAST_COMMIT = 2

    def __bool__(self):
        return self != GitDiff.NO_CHANGE 


def get_last_commit(dir):
    rev_file_name = path.join(dir, "last_modified.txt")
    last_commit = None
    try:
        with open(rev_file_name) as f:
            last_commit = next(f)
    except FileNotFoundError as f:
        pass
    return last_commit


def run_command_or_exit_with_error(command: List[str], **kwargs):
    error_msg = f"\n⛔ Error running command:\n\n    {' '.join(command)}\n"

    try:
        result = subprocess.run(command, capture_output=True, **kwargs)
    except FileNotFoundError as e:
        log_error()
        print(error_msg)
        print(f"      No such file: {command[0]}")
        sys.exit(1)
        
    if result.returncode != 0:
        log_error()
        print(error_msg)
        for pipe in [result.stdout, result.stderr]:
            for line in pipe.decode("utf-8").splitlines():
                if line.startswith("Gtk-Message:"):
                  continue
                print(line)
        sys.exit(1)


def get_most_recent_commit():
    result = subprocess.run(["git", "log", "-n", "1", "--pretty=format:%H"], capture_output=True)
    if result.returncode != 0:
        print("\nError getting most recent git commit")
        sys.exit(1)

    return result.stdout.decode("utf-8").strip()


def has_changed_since(dir, last_commit):
    if not last_commit:
        return GitDiff.NO_LAST_COMMIT
    else:
        git_status = subprocess.run(["git", "diff", "--quiet", last_commit, "--", dir])
        has_changed = git_status.returncode != 0
        if has_changed:
            return GitDiff.CHANGE_SINCE_LAST_COMMIT
        return GitDiff.NO_CHANGE


def run_kikit_commad(*command):
    # If you have kicad installed normally, you can just run `kikit` here.
    # My kicad is installed with flatpak so I have to install and run kikit
    # inside the flatpak environment
    run_command_or_exit_with_error(
        ["flatpak", "run", "--branch=stable", "--arch=x86_64", "--command=python3", "org.kicad.KiCad", "-c", "from kikit.ui import cli; cli()", *command],
    )

def run_kicad_cli_commad(*command):
    # If you have kicad installed normally, you can just run `kicad-cli` here.
    # My kicad is installed with flatpak so I have to install and run kikit
    # inside the flatpak environment
    run_command_or_exit_with_error(
        ["flatpak", "run", "--branch=stable", "--arch=x86_64", "--command=kicad-cli", "org.kicad.KiCad", *command],
    )


def log(indent, icon, msg, wait=False):
    msg = "".join([
        " " * (indent * 2),
        icon,
        " ",
        f"{msg}...".ljust(45 - indent * 2, ".") if wait else msg,
        " "
    ])
    if wait:
        print(msg, end="")
    else:
        print(msg)

def log_ok():
    print("✅ done")

def log_error():
    print("⛔ error")
 

def run_ibom_commad(*command):
    # See instructions for installing and calling generate_interactive_bom.py here: https://github.com/openscopeproject/InteractiveHtmlBom/wiki/Usage
    # If you have kicad installed normally, this should be simpler, but
    # my kicad is installed with flatpak so I have to run the script from
    # inside the flatpak environment
    path_to_generate_bom_script = "../InteractiveHtmlBom/InteractiveHtmlBom/generate_interactive_bom.py"
    run_command_or_exit_with_error(
        ["flatpak", "run", "--branch=stable", "--arch=x86_64", f"--command={path_to_generate_bom_script}", "org.kicad.KiCad", *command],
    )


def build_manual(name, output_dir, last_commit):
    manual_svg = path.abspath(path.join("modules", name, "docs", f"{to_snake_case(name)}_manual.svg"))
    if not path.exists(manual_svg):
        return
    if not has_changed_since(manual_svg, last_commit):
        return

    log(1, "🖨️ ", f"Building manual PDF for {name}", True)
    output_file = path.abspath(path.join(output_dir, f"{to_snake_case(name)}.pdf"))
    result = run_command_or_exit_with_error(
        ["inkscape", f"--actions=export-filename:{output_file};export-do", manual_svg],
    )
    log_ok()


def build_kicad_project(src_dir, output_dir, pcb_name, last_commit):
    pcb_file = path.join(src_dir, f"{pcb_name}.kicad_pcb")
    if not path.exists(pcb_file):
        return
    if not has_changed_since(src_dir, last_commit):
        return
    log(1, "⚙️ ", f"Building KiCad project for {pcb_name}:")

    log(2, "🛠️ ", "Exporting GERBERs:")
    # should be tempfile.mkdtemp() but kicad doesn't seem to work with /tmp files
    tmpdir = "tmp"
    for fab_flavor in ["jlcpcb", "pcbway"]:
        log(3, ">>", f"{fab_flavor}", True)
        result = run_kikit_commad("fab", fab_flavor, pcb_file, tmpdir, "--no-drc")
        gerber_zip = path.join(output_dir, f"{pcb_name}_{fab_flavor}.zip")
        os.replace(path.join(tmpdir, "gerbers.zip"), gerber_zip)
        shutil.rmtree(tmpdir)
        log_ok()

    if "faceplate" not in pcb_name:
        # Export HTML BOM
        log(2, "📑", "Exporting interactive BOM", True)
        output_dir_rel = os.path.realpath(output_dir)
        run_ibom_commad("--no-browser", f"--dest-dir={output_dir_rel}", "--name-format=%f_interactive_bom", "--blacklist=G*", pcb_file)
        log_ok()

        # Export schematic PDF
        schematic_file = path.join(src_dir, f"{pcb_name}.kicad_sch")
        sch_pdf_output = path.join(output_dir, f"{pcb_name}_schematic.pdf")
        log(2, "📝", "Exporting schematic", True)
        run_kicad_cli_commad("sch", "export", "pdf", "--no-background-color", "--output", sch_pdf_output, schematic_file)
        log_ok()


def to_snake_case(text):
    text = text.replace(" ", "")
    text = re.sub('(.)([A-Z][a-z]+)', r'\1_\2', text)
    return re.sub('([a-z0-9])([A-Z])', r'\1_\2', text).lower()


def build_faceplate(name, output_dir, last_commit):
    build_script = path.join("modules", name, "Faceplate", f"make_{to_snake_case(name)}_faceplate.py")
    if not has_changed_since(build_script, last_commit):
        return
    log(1, "🤖", "Building faceplate SVG", True)
    faceplate_file = to_snake_case(name)
    run_command_or_exit_with_error(
        [
            "python3",
            path.basename(build_script),
            "-o",
            path.abspath(path.join(output_dir, f"{to_snake_case(name)}_faceplate.svg")),
        ],
        cwd=path.dirname(build_script)
    )
    log_ok()


def build_rust_firmware(name: str, output_dir: str, last_commit: str):
    firmware_dir = path.join("modules", name, "Firmware")
    if not has_changed_since(firmware_dir, last_commit):
        return
    log(1, "🦀", "Building firmware", True)
    env = os.environ.copy()
    env["RUSTFLAGS"] = "-Zlocation-detail=none"
    run_command_or_exit_with_error(
        ["cargo", "build", "--release"],
        env=env,
        cwd=firmware_dir,
    )

    firmware_name = f"fm-{name.lower().replace('_', '-')}"
    elf_file = path.join(firmware_dir, "target", "avr-atmega328p", "release", f"{firmware_name}.elf")
    hex_file = path.join(output_dir, f"{firmware_name}.hex")

    run_command_or_exit_with_error(
        ["avr-objcopy", "-O", "ihex", elf_file, hex_file],
        env={"RUSTFLAGS": "-Zlocation-detail=none"},
    )
    log_ok()


def build(name, output_dir):
    dir = path.join("modules", name)
    output_dir = path.join(output_dir, name)
    last_commit = get_last_commit(output_dir)
    change = has_changed_since(dir, last_commit)
    if change == GitDiff.NO_CHANGE:
        return

    if change == GitDiff.NO_LAST_COMMIT:
        log(0, "📦", f"Building {name} (no last commit)")
    else:
        log(0, "📦", f"Building {name} (last built from #{last_commit[:7]})")

    current_commit = get_most_recent_commit()
    if has_changed_since(dir, current_commit):
        log(1, "⚠️ ", "Warning: building from untracked changes")

    os.makedirs(output_dir, exist_ok=True)

    for pcb_name in [
        f"{name.lower()}_pcb",
        f"{name.lower()}_front_pcb",
        f"{name.lower()}_back_pcb",
        f"{name.lower()}_faceplate",
        f"{name.lower()}_faceplate_pcb",
    ]:
        kicad_proj_dir = path.join(path.abspath(dir), "PCBs", pcb_name)
        if path.isdir(kicad_proj_dir):
            build_kicad_project(kicad_proj_dir, output_dir, pcb_name, last_commit)

    build_faceplate(name, output_dir, last_commit)

    cargo_toml = path.join(dir, "Firmware", "Cargo.toml")
    if path.exists(cargo_toml):
        build_rust_firmware(name, output_dir, last_commit)

    build_manual(name, output_dir, last_commit)

    rev_file_name = path.join(output_dir, "last_modified.txt")
    with open(rev_file_name, "w") as f:
        f.write(current_commit)
    


if __name__ == "__main__":
    output_dir = "../fm-artifacts"
    build("Clock", output_dir)
    build("Mixer", output_dir)
    # build("DiodeDistortion", output_dir)

