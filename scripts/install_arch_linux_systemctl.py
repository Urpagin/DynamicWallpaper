import subprocess
import os
import shlex


def run_cmd(cmd: str) -> tuple[str, str]:
    """Runs a command with the OS' terminal using shlex for proper tokenization."""
    cmd_tokens = shlex.split(cmd)  # Use shlex.split to correctly tokenize the command
    result = subprocess.run(cmd_tokens, capture_output=True, text=True)

    if result.stderr:
        print(f"ERROR: '{cmd}' -> {result.stderr}")

    return result.stdout, result.stderr


def get_install_path() -> str:
    """
    Returs the install path for the program.
    """
    home_directory = os.environ.get('HOME')
    install_dir = f'{home_directory}/dyn_wallpapers'

    try:
        os.makedirs(install_dir)
        print(f"Created directory '{install_dir}'.")
    except OSError:
        print(f"Path '{install_dir}' already exists. Aborting install.")
        exit(0)


    return install_dir


def clone_repo() -> None:
    install_path = get_install_path()
    run_cmd(f"git clone https://github.com/Urpagin/DynamicWallpaper.git {install_path}")




def main() -> None:
    clone_repo()

if __name__ == '__main__':
    main()
