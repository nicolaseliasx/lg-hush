# lg-hush

A quiet, pure-black Home for rooted LG webOS TVs, with a built-in, reversible debloat.

lg-hush is a derivative of [Luma](https://github.com/ecklf/lg-luma) by ecklf and the Luma
contributors (MIT). The launcher, renderer and most of the code are theirs; this project adds a
default look and a TV helper on top. See [Credits](#credits).

> [!WARNING]
> This needs a **rooted** TV and changes how the TV's Home works. Rooting voids warranties and
> can break your TV. You use everything here **at your own risk**. Nothing in this repository
> roots a TV, and nothing writes to the TV's system partitions (see [what it changes](#what-it-changes)).

## What it is

- **Home as a black launcher.** A pure black background, no greeting, date, weather or webhook
  shortcuts, and the favorites on a dock in a slightly lighter black, like Apple TV's. Apps, inputs
  and recent apps behave like upstream Luma. The app icon is a plain "H", without the Luma logo.
- **`luma-helper`**, a small root-only helper packaged with the app:
  - `diagnose` reads what your TV exposes and whether the Home replacement can be enabled. It changes nothing.
  - `tune` is the debloat: it switches off LG preloads and some services and blocks a few LG ad
    hosts, all in RAM and reapplied at boot. `tune restore` puts LG's own settings back.
- **Faster cold boot and a Home without the cursor** when returning from an app.
- **Continue Watching** (optional): the titles in progress in your Stremio library, with episode,
  time left and a progress bar, between the tabs and Favorites. Sign in once with
  `scripts/stremio-login.sh`. Needs Stremio on the TV to open a title.
- **Power off instead of the LG screensaver** when Luma is the Home.
- A fork identity: app id `cc.ncls.luma`, update checks disabled (this project has no update feed yet).

It is **not** a promise of better performance or of removing all data collection by LG. Measure on
your own TV.

## Tested on

| TV               | Firmware    | Result                                                                            |
| ---------------- | ----------- | --------------------------------------------------------------------------------- |
| LG `55UP751C0SF` | webOS 6.5.3 | Home replacement, debloat, boot hook, Continue Watching: used daily by the author |

Anything else is untested. The upstream project reports Home replacement on webOS 5, 6, 25 and 26;
see the [upstream README](docs/luma-upstream.md) for its device table. Run `luma-helper diagnose`
before enabling anything.

## Install

Short version, from a computer on the same network as your rooted TV:

```sh
scripts/install-tv.sh TV_IP dist/cc.ncls.luma_4.0.0_arm.ipk
```

The full guide, including prerequisites, enabling the Home replacement, the debloat and how to
undo each step, is in **[docs/install.md](docs/install.md)** ([pt-BR](docs/install.pt-BR.md)).
Prebuilt packages are on the [Releases](https://github.com/nicolaseliasx/lg-hush/releases) page, with checksums.

## What it changes

| Step                     | What happens                                                                        | How to undo                                       |
| ------------------------ | ----------------------------------------------------------------------------------- | ------------------------------------------------- |
| Install the IPK          | Adds the app `cc.ncls.luma`                                                         | Uninstall it (disable the Home replacement first) |
| `home.sh enable`         | Bind-mounts Luma over LG's Home and adds a boot hook. LG's files stay underneath.   | `home.sh disable`                                 |
| `luma-helper tune apply` | RAM-only mounts over some LG services and ad hosts; LG's preload list is saved once | `luma-helper tune restore`                        |

A reboot without the boot hook gives LG's Home back. Details and the exact lists are in
[development.md](development.md) ("Optional native Home integration", "Performance debloat").

## Stremio Cards (not included)

The Continue Watching row can open a title directly if a second Stremio build that accepts launch
parameters is installed as `cc.ncls.stremiocards`. That build is a third-party project without a
licence, so it is **not** part of this repository. Without it, a card opens the official Stremio app
and shows the title to search for.

## Build

```sh
nix develop
pnpm install --frozen-lockfile
pnpm sdk
export WEBOS_NDK="$PWD/.build/toolchain/arm-webos-linux-gnueabi_sdk-buildroot"
pnpm run package
```

Output:

```text
dist/cc.ncls.luma_4.0.0_arm.ipk
dist/sha256sums
dist/repo.json
```

Desktop preview (no TV needed): `pnpm install --frozen-lockfile && pnpm assets && pnpm dev`.
See [development.md](development.md) for the source layout, tests and diagnostics.

## Documents

- [Install guide](docs/install.md) · [pt-BR](docs/install.pt-BR.md)
- [Project plans and measurements](docs/plans/README.md) (in Portuguese)
- [Upstream Luma README](docs/luma-upstream.md)
- [Development guide](development.md) · [Releasing](docs/releasing.md) · [Changelog](CHANGELOG.md)

## Credits

- **[Luma](https://github.com/ecklf/lg-luma)** by ecklf and contributors, MIT: the launcher this is based on.
- **[webosbrew](https://www.webosbrew.org/)** and the Homebrew Channel, which make rooted TV apps possible.
- **[SDL-webOS](https://github.com/webosbrew/SDL-webOS)**, **[Cinemeta](https://v3-cinemeta.strem.io/)** and the other
  third-party sources listed in [`assets/licenses`](assets/licenses).
- [Glasshouse](https://github.com/rorygallagher2024/glasshouse) and CleanYourTV, studied as references for the debloat.

lg-hush is not affiliated with LG, Stremio, Apple or any other brand shown in its UI.

## License

MIT. The original notice is kept; see [license](license).
