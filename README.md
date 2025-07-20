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


# Installation

## Server (Docker)

> [!TIP]
> For a non-Docker installation, the user should follow the standard Rust building procedure inside of the `server` directory. <a href="https://doc.rust-lang.org/book/ch01-03-hello-cargo.html" target="_blank" rel="noopener">Click here for more information</a>.

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


## Client

This part is a little more hazardous. You can try running the `scripts/install_arch_linux_systemctl.py` script; this program *might* work on your machine.  
It requires **systemd** and has been tried only on **Arch Linux**.  


> [!NOTE]  
> **The recommended method is as follows:**

I recommend manually building the `client` component (see the **TIP** on how to do that under the [Server (Docker) section](#server-docker)).  
Then copy and populate the `scripts/update_wallpaper_feh.sh` wherever you'd like and make it autostart by executing the script in
a startup file like `~/.xinitrx`, `~/.xsession` if you're on X11 or place `exec-once = path/to/script.sh &` in `~/.config/hypr/hyprland.conf` if you're using Wayland + Hyprland.


Otherwise, you can directly interact with the produced binary using this argument convention:
```bash
./client \
  --endpoint "<your endpoint, e.g., https://wallpapers.yourdomain.com>" \
  --directory "<directory path where to store the downloaded images>" \
  --user "<(optional) NGINX simple auth>" \
  --password "<(optional) NGINX simple auth>"
```

# Security
I am a beginner, so I cannot guarantee that there are no security vulnerabilities.

This is one of the reasons I should add a password. On that note, adding a password-protected proxy (such as NGINX) would be an easier and simpler option. (That is what happens if you decide to run the server component with the provided `docker-compose.yml`; A simple password-protected proxy using NGINX is set up.)
