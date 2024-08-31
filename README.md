# DynamicWallpaper
A webui/sync program to change your wallpapers.

# Explanation

The server/webui part acts as the interface for any user to add/delete images.

The client/sync should be ran at intervals to keep the local images in-sync with the server, then use any wallpaper program (like feh) to put your wallpaper from the local collection.

## Simply Put

- You upload images on the webui (you can delete them too).

- You run the client part and images from the webui server will be synced to your local directory.

- Then you use your favorite wallpaper program to choose at random in your local synced image directory.

# Why?

- Learning opportunity.
- Allows multiple computers/friends to share the same collection of wallpapers.

# Security
I am but a beginner, I cannot guarantee that there is no exploits.

This is one of the reason I should add a password. On that note, adding some kind of password-protected proxy should be the lazier but simpler option (like NGINX).

# Installation

## Server

### Requirements

- htpasswd: `sudo apt-get install apache2-utils`
- Docker & Docker compose (see the Docker website for installation)

Clone the repo to begin with: `git clone https://github.com/Urpagin/DynamicWallpaper.git`

1. For the NGINX password-protected proxy be sure to generate the `htpasswd` file with:

    `htpasswd -c htpasswd <username>`

3. Start up the NGINX proxy main program:

    `sudo docker compose up --build -d`
> [!TIP]  
> The `--build` argument rebuilds the container so that code updates will be reflected onto the container.

4. Visit `127.0.0.1:<port>` (the default port is 8080 inside `docker-compose.yml`) to access the app.

> [!NOTE]  
> Once you've started the app via Docker compose, the `wallpapers_server` directory will contain the wallpapers.

