# arigato!

`arigato` is a very barebones Rust framework for creating and serving a 9p
filesystem. Running on port `564` is customary; you may need to use `setcap`
or `iptables` or something to route traffic to your binary.

This currently only supports `9P2000.u`; but that may change in the future.
This uses nightly-only features and isn't documented. Most of this was written
fairly quickly and carelessly to implement `debugfs` but I may keep this up
to date over the next few years.
