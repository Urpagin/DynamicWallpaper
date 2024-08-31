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

- Docker & Docker compose (see the Docker website for installation)

### Steps

1. Clone the repo
    git clone `git clone https://github.com/Urpagin/DynamicWallpaper.git`

2. Create file `.env` at the same level as `docker-compose.yml` and add those varaibles:
```env
PORT=<exposed port>
NGINX_USER=<user for webui auth>
NGINX_PASSWORD=<password for webui auth>
```

3. Start up the containers
   `sudo docker compose up -d --build` 
> [!TIP]  
> The `--build` argument rebuilds the container so that code updates will be reflected onto the container.

4. Visit `127.0.0.1:<PORT>` to access the app.
> [!NOTE]  
> Once you've started the app via Docker compose, the `wallpapers_server` directory will contain the wallpapers.
