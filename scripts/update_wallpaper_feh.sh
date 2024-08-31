#!/bin/bash

# 1. Install dependencies: feh
# 2. Download or compile client side program and put the binary next to this script. (chmod +x it too)
# 3. Populate the variables
# 4. Run script (you may run it at your computer's startup)

# BINARY NAME (the client-side binary's name)
BINARY_FILE_NAME=client_x86_64_linux

# SERVER URL
ENDPOINT=https://domain.ext

# DIRECTORY (where the images will be saved)
WALLPAPERS_PATH=./wallpapers

# NGINX
USER=user
PASSWORD=password



# Fetch wallpapers
./"$BINARY_FILE_NAME" --endpoint "$ENDPOINT" --directory "$WALLPAPERS_PATH" --user "$USER" --password "$PASSWORD"

# Set a random wallpaper
feh --bg-fill --randomize "$WALLPAPERS_PATH"/*

