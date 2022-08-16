# fake-wmic

This repo builds a fake wmic.exe to cheat [@dropb/diskinfo](https://github.com/kukhariev/diskinfo) in wine environment. It only implements a extremely naive parser, and only accepts "logicaldisk" as its command.

## Why?

I want to run a close-sourced Electron app in wine. It mostly works but one of its crucial functionality requires "disk size > 2GB", which uses `@dropb/diskinfo` to [detect free disk space](https://github.com/kukhariev/diskinfo/blob/master/src/win32.ts).

Sadly, wine does not implement `wmic` yet:

```
Z:\home\username>wmic
Error: Command line not supported
```

So, why don't we implement a naive `wmic` ourselves and trick it?

## Cross-compile build

1. `rustup target add x86_64-pc-windows-gnu`
2. Install mingw-w64 gcc toolchain
3. `cargo build --target x86_64-pc-windows-gnu`
4. The `fake-wmic.exe` is in `target/x86_64-pc-windows-gnu/debug/fake-wmic.exe`. Bring it to elsewhere.

You DON'T NEED to replace C:\Windows\System32\wmic.exe in wine. As it directly calls `child_process.execFile` with `WMIC`, just rename it to `wmic.exe` and put it in the same folder as the Electron/Nodejs executable. You DON'T NEED to modify winecfg libraries settings.

## Example

```console
> cargo run -- logicaldisk where "not size=null and name = 'c:'" get name, drivetype, size, freespace
drivetype       freespace       name    size
3       1000000000000   C:      1000000000000
> cargo run -- logicaldisk where "not size=null" get name, drivetype, size, freespace
drivetype       freespace       name    size
3       1000000000000   C:      1000000000000
3       1000000000000   Z:      1000000000000
> cargo run
wmic:root\cli>logicaldisk where "not size=null and name = 'c:'" get name, drivetype, size, freespace
drivetype       freespace       name    size
3       1000000000000   C:      1000000000000
wmic:root\cli>logicaldisk where "not size=null" get name, drivetype, size, freespace
drivetype       freespace       name    size
3       1000000000000   C:      1000000000000
3       1000000000000   Z:      1000000000000
wmic:root\cli>quit
```
