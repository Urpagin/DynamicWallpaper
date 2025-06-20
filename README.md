# DynamicWallpaper
 A web UI and synchronization program to change your wallpapers.
 
(Well, `rsync` is simpler and better! Anyway, it was fun to program DynamicWallpaper myself.)

# Explanation

The server/web UI component acts as the interface for users to **add** or **delete** images.

The client/sync should be run at intervals to keep the local images in sync with the server. Then, use any wallpaper program (such as feh) to set your wallpaper from the local collection.

## Simply Put

- You upload images to the web UI (you can delete them too).

- You run the client component, and images from the web UI server will be synced to your local directory.

- Then, use your favorite wallpaper program to select a random image from your locally synced directory.

# Why?

- Learning opportunity.
- Allows multiple computers and friends to share the same collection of wallpapers.

# Security
I am a beginner, so I cannot guarantee that there are no security vulnerabilities.

This is one of the reasons I should add a password. On that note, adding a password-protected proxy (such as NGINX) would be an easier and simpler option.

# Installation

## Server

### Requirements

- Docker and Docker Compose (see the Docker website for installation).

### Steps

1. Clone the repo
   
   `git clone https://github.com/Urpagin/DynamicWallpaper.git`

3. Create a .env file at the same level as docker-compose.yml and add these variables:
```env
PORT=<exposed port>
NGINX_USER=<user for web UI auth>
NGINX_PASSWORD=<password for webui auth>
```

3. Start the containers:
   `sudo docker compose up -d --build` 
> [!TIP]  
> The --build argument rebuilds the container so that code updates are reflected in the container.

4. Visit `127.0.0.1:<PORT>` to access the app.
> [!NOTE]  
>  Once you have started the app via Docker Compose, the `wallpapers_server` directory will contain the wallpapers.
