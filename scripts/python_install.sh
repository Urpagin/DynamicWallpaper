#!/bin/bash

# If any command fails, exit the script (suupernice)
set -e

if [[ "$USER" != "root" ]]; then
    echo 'You must run this script as root. Aborting.'
    exit -1
fi


# Function to check which shell is currently in use
detect_shell() {
    case "$SHELL" in
        */bash)
            echo "bash"
            ;;
        */zsh)
            echo "zsh"
            ;;
        */fish)
            echo "fish"
            ;;
        *)
            echo "unknown"
            ;;
    esac
}


# Function to source the virtual environment based on the shell
source_venv() {
    local shell_type=$(detect_shell)
    if [[ "$shell_type" == "bash" || "$shell_type" == "zsh" ]]; then
        echo "Sourcing virtual environment using $shell_type..."
        source venv/bin/activate
    elif [[ "$shell_type" == "fish" ]]; then
        echo "Sourcing virtual environment using fish shell..."
        source venv/bin/activate.fish
    else
        echo "ERROR: Unsupported or unknown shell. Please manually source the virtual environment."
        exit 1
    fi
}

# Function to check if the operating system is Arch Linux
check_os() {
    if [[ -f /etc/arch-release ]]; then
        echo "Operating System: Arch Linux"
    else
        echo "ERROR: This script is intended for Arch Linux only."
        exit 1
    fi
}

install_dependencies() {
    sudo pacman -S git python jq feh --noconfirm
}

# Function to check if a command exists
check_command() {
    local cmd="$1"
    if ! command -v "$cmd" &>/dev/null; then
        echo "ERROR: '$cmd' is not installed."
        echo "\n\nPLEASE EXEXUTE"
        exit 1
    else
        echo "'$cmd' is installed."
    fi
}

# Main function to check all requirements
check_requirements() {
    check_os
    check_command "git"
    check_command "python3"
    check_command "jq"
    check_command "feh"

}

# Run the checks
check_requirements

# Run installation
if [[ ! -d "venv" ]]; then
    echo 'Creating Python virtualenv, this may take a few seconds, please wait...'
    python3 -m venv venv
    echo 'Created Python virtualenv.'
else
    echo 'Python virtualenv already exists. Skipping creation.'
fi

# Source the virtual environment
source_venv

echo 'Running python install script'
python3 install_arch_linux_systemctl.py


