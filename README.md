# KOONTUNE Downloader (yar)

This program will create a music library based on a Library file.

# yarb

Builds a single yaml file from multiple files. See the [phonkhub repo](https://github.com/phonkhub/db) for an example.

# yarcamp

Builds a album file from a bandcamp or soundcloud url.


# How to use for Jon

0. Open Terminal

1. Run these commands

```sh
brew install rust
git clone https://github.com/k2on/yar
cd yar
sudo ./install
sudo ./mk-sync
```

2. Add an album

Create a folder in /Users/jon/.music/artists called "backwhen"

Download [this file](https://raw.githubusercontent.com/phonkhub/db/main/artists/backwhen/sensations.yml) into that folder, called "sensations.yml"

3. Go back to terminal and run `tunesync`

4. Listen to ur music

You will now have the album in your media/music folder
