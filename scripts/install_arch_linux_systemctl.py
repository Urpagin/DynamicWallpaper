import subprocess
from pathlib import Path
import urllib.request
import urllib.error
import os
import shlex


def run_cmd(cmd: str) -> tuple[str, str]:
    """Runs a command with the OS' terminal using shlex for proper tokenization."""
    cmd_tokens = shlex.split(cmd)  # Use shlex.split to correctly tokenize the command
    result = subprocess.run(cmd_tokens, capture_output=True, text=True)

    if result.stderr:
        print(f"ERROR: '{cmd}' -> {result.stderr}")

    return result.stdout, result.stderr


def get_user() -> str:
    """
    Returns the name of the user that ran the script with sudo permissions.
    """
    user: str | None = os.environ.get('SUDO_USER')
    if user is None:
        print('Not running as root. Aborting.')
        exit(-1)
    return user


def get_install_path() -> str:
    """
    Returs the install path for the program.
    """
    home_directory = str(Path(f"/home/{get_user()}").expanduser())
    print(home_directory)
    install_dir = f'{home_directory}/dyn_wallpapers'

    try:
        # Try making the install_dir and install_dir/wallpapers directories.
        os.makedirs(os.path.join(install_dir, 'wallpapers'))
        print(f"Created directory '{install_dir}'.")
    except OSError:
        print(f"Path '{install_dir}' already exists. Aborting install.")
        exit(-1)


    return install_dir


def clone_repo() -> None:
    install_path = get_install_path()
    run_cmd(f"git clone https://github.com/Urpagin/DynamicWallpaper.git {install_path}")


def ping_url(url: str) -> bool:
    """
    Pings the given `url` to check if the server responds. Returns True if the server responds with any status code, otherwise False.
    """
    try:
        # Create a request object with a user-agent header to mimic a browser
        request = urllib.request.Request(url, method='HEAD')
        request.add_header('User-Agent', 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3')
        
        # Perform the request
        response = urllib.request.urlopen(request)
        print(f"Server responded with status code: {response.status}")
        return True
    except urllib.error.HTTPError as e:
        # If the server responds with an HTTP error, still consider it reachable
        print(f"Server responded with status code: {e.code}")
        return True
    except urllib.error.URLError as e:
        # If the server is unreachable, print the reason and return False
        print(f"Failed to reach the server. Reason: {e.reason}")
        return False


def handle_url_validity(url: str) -> None:
    """
    Exits the program if the URL does not respond to a HTTP GET request. Logs requests too.
    """
    if not url.startswith('https://') and not url.startswith('http://'):
        print(f"The URL '{url}' does not start with 'https://' or 'http://'. Aborting.")
        exit(-1)
    print('Checking provided endpoint URL...')
    if not ping_url(url):
        exit(-1)
    print('URL successfully validated.')


def get_user_input(prompt: str) -> str:
    """
    Propts the user with the input prompt and return his response.
    """
    while True:
        user_input: str = input(f'{prompt}\n-> ').strip()
        if user_input:
            return user_input
        print('Invalid input, please try again.\n')


def download_release(directory: str, url: str) -> str:
    """
    Downloads a release file from the given `url` URL and saves it into `directory`. Returns the filename.
    """
    # Extract the filename from the URL
    filename: str = os.path.basename(url)
    full_path: str = os.path.join(directory, filename)
    
    try:
        # Download the file from the URL and save it locally
        urllib.request.urlretrieve(url, full_path)
        print(f"File successfully downloaded to: {full_path}")
        return filename
    except urllib.error.URLError as e:
        print(f"Failed to download the file. Reason: {e.reason}")
        exit(-1)


def create_update_script(install_path: str, script_filename: str) -> str:
    """
    Creates the update_wallpaper.sh file, populates it.
    """
    download_url: str = get_user_input('Go in the GitHub releases for this repo, right click on the right release file and copy the download link, paste it here')
    handle_url_validity(download_url)
    program_filename: str = download_release(install_path, download_url)

    endpoint: str = get_user_input('Provide the server endpoint URL (https://...)')
    handle_url_validity(endpoint)

    nginx_user: str = get_user_input('Provide the NGINX username')
    nginx_password: str = get_user_input('Provide the NGINX password')



    program_path: str = os.path.join(install_path, program_filename)
    run_cmd(f'chmod +x {program_path}')
    print('Added execution permissions to the program binary.')

    file_content: str = f"""#!/bin/bash
set -e

# 1. Install dependencies: feh
# 2. Download or compile client side program and put the binary next to this script. (chmod +x it too)
# 3. Populate the variables
# 4. Run script (you may run it at your computer's startup)

# BINARY NAME (the client-side binary's name)
BINARY_FILE_NAME={program_path}

# SERVER URL
ENDPOINT={endpoint}

# DIRECTORY (where the images will be saved)
WALLPAPERS_PATH={os.path.join(install_path, 'wallpapers')}

# NGINX
USER={nginx_user}
PASSWORD={nginx_password}



# Fetch wallpapers
"$BINARY_FILE_NAME" --endpoint "$ENDPOINT" --directory "$WALLPAPERS_PATH" --user "$USER" --password "$PASSWORD"

# Set a random wallpaper
feh --bg-fill --randomize "$WALLPAPERS_PATH"/*

"""

    print(f'File content: \n\n{file_content}\n\n')

    script_path: str = os.path.join(install_path, script_filename)


    with open(script_path, 'w', encoding='utf-8') as file:
        file.write(file_content)
    print(f"Wrote script file at '{script_path}'.")

    run_cmd(f'chmod +x {script_path}')
    print('Added execution permissions to script path.')

    return script_path


def run_script_test(script_path: str):

    """
    Runs the script a first time and asks the user if the wallpaper changed.
    """

    print('\n\n\n\n\n')
    print('Program installation is completed.\n\nPress [ENTER] to make a test run, and verify that your wallpaper changed.\n\nIf your wallpaper did not change, the installation cannot continue further.')
    input('\nPRESS ENTER TO MAKE A TEST RUN...')

    print(f"Running script '{script_path}'...")
    print('This may take a few seconds...')

    run_cmd(script_path)

    print('\n\n')
    resp: str = get_user_input('Did your wallpaper change? (N/y)').lower()
    if resp == 'n' or resp == 'no':
        print('Abording, cannot continue installation if the program did not work.')
        exit(-1)
    else:
        print('Continuing installation. Installing program at computer startup.')





def systemctl_program_setup(install_path: str, script_filename: str):
    """
    Installs the program to run at computer start with systemctl.
    """

    # Get the current username from environment variables
    computer_username: str = get_user()

    # Construct the full path to the script
    script_full_path = os.path.join(install_path, script_filename)

    # Define the content of the systemd service file
    service_file_content: str = f"""[Unit]
Description=Update and set wallpaper by Urpagin
After=network.target

[Service]
Type=oneshot
ExecStart={script_full_path}
WorkingDirectory={install_path}
User={computer_username}
Environment=DISPLAY=:0
Environment=XAUTHORITY=/home/{computer_username}/.Xauthority

[Install]
WantedBy=multi-user.target"""

    # Define the path to the service file
    service_file_path = '/etc/systemd/system/update-wallpaper.service'

    # Write the service file to the /etc/systemd/system directory
    try:
        with open(service_file_path, 'w') as service_file:
            service_file.write(service_file_content)
        print(f"Service file created at {service_file_path}")
    except PermissionError:
        print(f"Permission denied: You need to run this script with sudo or as root to write to {service_file_path}.")
        exit(-1)

    # Reload systemd daemon and enable the service
    run_cmd('sudo systemctl daemon-reload')
    run_cmd('sudo systemctl enable update-wallpaper.service')


def main() -> None:
    SCRIPT_FILENAME: str = 'update_wallpapers.sh'

    if None is os.environ.get('USER') or os.environ.get('USER') != 'root':
        print('You must run this script as root. Aborting')
        exit(-1)


    install_path: str = get_install_path()
    script_path = create_update_script(install_path, SCRIPT_FILENAME)

    # Add permissions for the user with the install directory
    print(f'Adding permissions for user \'{get_user()}\' to directory \'{install_path}\'')
    run_cmd(f'sudo chown -R {get_user()}:{get_user()} {install_path}')

    run_script_test(script_path)

    systemctl_program_setup(install_path, SCRIPT_FILENAME)

if __name__ == '__main__':
    main()
