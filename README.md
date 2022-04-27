## eon mod downloader
`emd` is a small program to conveniently install mods of a given minecraft version. Currently it only works with [Modrinth](https://modrinth.com/), but most fabric mods can be found from there.

## Usage
- Within the same folder as the executable (emd.exe) you must have a [mods.toml](./mods.toml) file
- The [mods.toml](./mods.toml) is the configuration file where you can set the mod list to download in addition to a few other options
    - To add a mod get the name of a mod from the url of a mods Modrinth page and add it to the list labelled `modrinth` like the other mods in the [example config file](mods.toml)
    - To change version set `minecraft-version` to your preferred version using the following formatting `minecraft-version = "1.18.2"`
    - To set the maximum amount of mods downloading at the same time set `simoltaneous-downloads` to a non-negative number. Make sure the number is sensible for example 4. If the given number is too big it might slow down your computer and slow down the download speed of all the mods.
- Once you have your preferred mods defined in the configuration file just double click the executable and the mods should begin installing in the same folder as the executable

### TODO
- prevent the user from setting simoltaneous-downloads to absurd numbers
- add other source sites:
    - github
    - (maybe) curseforge
