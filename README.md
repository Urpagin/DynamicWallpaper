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

todo!()

