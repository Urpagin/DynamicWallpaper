#!/bin/bash

# 1. Install dependencies: feh
# 2. Download or compile client side program and put the binary next to this script. (chmod +x it too)
# 3. Populate the variables
# 4. Run script (you may run it at your computer's startup)

# You may call this script in your ~/.xinitrc or ~/.xsession
# or in your ~/.config/hypr/hyprland.conf if you use Hyprland: exec-once = this-script &

# BINARY PATH (the client-side binary's path)
BINARY_FILE_NAME='./client_x86_64_linux'

# SERVER URL
ENDPOINT='https://yourdomain.ext'

# DIRECTORY (where the images will be saved)
WALLPAPERS_PATH='./somewhere/change/me'

# NGINX
USER='user'
PASSWORD='password'


# Remove --user and --password if you don't have NGINX simple auth
# Fetch wallpapers
./"$BINARY_FILE_NAME" --endpoint "$ENDPOINT" --directory "$WALLPAPERS_PATH" --user "$USER" --password "$PASSWORD" &

# Select random wallpaper
wallpaper=$(find "$WALLPAPERS_PATH" -type f | shuf -n 1)

# Set a random wallpaper
feh --bg-max "$wallpaper"

