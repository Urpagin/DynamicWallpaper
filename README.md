# DynamicWallpaper

A web UI and synchronization program to change your wallpapers.

(Yes, `rsync` is simpler and better. Anyway, it was fun to program DynamicWallpaper.)

## Explanation 🧩

The server and web UI let users **add** and **delete** images.

The client sync tool should be run at intervals to keep local images in sync with the server. Then use any wallpaper program (such as `feh`) to set your wallpaper from the local collection.

## Simply Put ✨

* Upload images through the web UI. You can delete them too.
* Run the client. Images from the server are synced to a local directory.
* Use your favorite wallpaper tool to pick a random image from that local directory.

## Why ❓

* Learning opportunity.
* Lets multiple computers and friends share the same wallpaper collection.

# Installation 🧰

## Server (Docker) 🐳

> [!TIP]
> For a non-Docker installation, follow the standard Rust build workflow inside the `server` directory. <a href="https://doc.rust-lang.org/book/ch01-03-hello-cargo.html" target="_blank" rel="noopener">Click here for more information</a>.

### Requirements

* Docker and Docker Compose

### Steps

1. Clone the repo

   ```bash
   git clone https://github.com/Urpagin/DynamicWallpaper.git
   ```

2. Create a `.env` file next to `docker-compose.yml` and add:

   ```env
   PORT=<exposed port>
   NGINX_USER=<user for web UI auth>
   NGINX_PASSWORD=<password for web UI auth>
   ```

3. Start the containers

   ```bash
   sudo docker compose up -d --build
   ```

> [!TIP]
> The `--build` flag rebuilds the image so code updates are reflected.

4. Open `http://127.0.0.1:<PORT>` to access the app.

> [!NOTE]
> Once the stack is running, the `wallpapers_server` directory will contain the wallpapers.

## Client 🖥️

This part is a little more hazardous. You can try running `scripts/install_arch_linux_systemctl.py`. This script requires **systemd** and has been tested only on **Arch Linux**.

> [!NOTE]
> **Recommended approach**
>
> Build the `client` component manually (see the tip in the [Server (Docker) section](#server-docker)). Then copy `scripts/update_wallpaper_feh.sh` somewhere convenient, make it executable, and have it autostart.
>
> ```bash
> cargo build --release -p client
> chmod +x /path/to/update_wallpaper_feh.sh
> ```
>
> On X11, invoke the script from a startup file like `~/.xinitrc` or `~/.xsession`. On Wayland with Hyprland, add a line like `exec-once = /path/to/update_wallpaper_feh.sh &` to `~/.config/hypr/hyprland.conf`.

You can also manually invoke the client binary directly with these arguments:

> [!NOTE]
> The example assumes you are protecting the server behind NGINX basic auth.

```bash
./client \
  --endpoint "https://wallpapers.yourdomain.com" \
  --directory "/path/to/downloaded/images" \
  --user "<optional basic auth user>" \
  --password "<optional basic auth password>"
```

## Security 🔒

I am a beginner, so I cannot guarantee there are no vulnerabilities.

The provided Docker Compose setup includes a simple password-protected proxy using NGINX.
