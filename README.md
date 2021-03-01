# rustone
A Minecraft server management program, written in Rust.

Rustone aims to provide a simple way to manage Minecraft servers - be it
from CLI, or straight from a Discord bot which calls an API to launch
a server on-demand.

Rustone currently integrates with [PaperMC](https://papermc.io), a high-performance Spigot fork.
There are plans to support BungeeCord, and other MC server software.

Rustone is divided into the core library - named `rustone` - and two current
frontends:
 - `rscmd`: command-line frontend
 - `rshttp`: provides an HTTP server with an API
 
The project started not too long ago, when people started to ask me to run
a server which should be available when they want to play. Because
I didn't want to keep my computer always on, I've written a basic server
management script, but it was still too slow to start up my computer and
type commands manually - and thus, `rshttp` was born.

Rustone is not close to any release yet, and the internal API is not stabilized.
The frontends should be more or less backwards compatible. *You should not use
Rustone for production. Yet.*
