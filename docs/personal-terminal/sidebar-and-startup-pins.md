# Compact terminal sidebar and permanent startup pins

The personal fork defaults to compact vertical tabs. Terminal rows show a single
line with a 16 px icon and 3 px top/bottom padding. Expanded mode remains available
from the sidebar display menu for additional metadata.

Use the existing tab menu to create a group, move tabs into it, rename it, or pin
it. Groups are collapsible folders. Pinning a group includes its terminal tabs.

Pinned terminal tabs are also saved as local startup bookmarks in
`permanent-pins.json` under the profile's config directory (Linux OSS:
`~/.config/warp-oss/permanent-pins.json`). On launch, bookmarks open together in a
window, with their group names, collapse state, colors, and saved startup commands.
Ordinary session restoration continues for other tabs. Restored terminal UUIDs
prevent a bookmarked terminal from opening twice.

Closing a pinned terminal temporarily leaves its bookmark. To stop it opening at
startup, unpin it before closing it. Unpinning a group also removes its closed
members from startup. Renaming a group updates the saved folder name. Bookmarks
use stable IDs, so equal tab names do not collide.

Launch-config tabs accept `group`, `pinned`, and optional `startup_pin_id` fields.
A group has `id` (UUID), `name`, `collapsed`, `pinned`, and optional `color`.
Launch configs retain group metadata when saved. Startup commands from a launch
config are retained when pinned; manually opened single-pane interactive SSH
connections are captured as reconnect commands. Other running shell commands are
not automatically rerun. Local bookmarks restore terminal layouts, not live
remote processes or coding-agent conversations. Use remote tmux when process
persistence is needed.

The school-laptop migration installs one fresh terminal per configured remote,
plus a local GRID terminal. It does not transfer private keys, terminal scrollback,
or the cmux daemon's running PTYs. Individual machine TOML configs remain usable
in stock Warp. Folder startup bookmarks require this fork.
