# Installing lg-hush on a webOS TV

[Português](install.pt-BR.md) · [Back to the README](../readme.md)

> [!WARNING]
> You need a **rooted** TV. This guide does not cover rooting; follow the
> [webosbrew rooting guide](https://www.webosbrew.org/rooting/) for your model and firmware. Everything
> below is **at your own risk**, and was tested on a single TV (see [Tested on](../readme.md#tested-on)).
> The install scripts have not been run against other models.

## 1. Prerequisites

- A rooted LG webOS TV with the **Homebrew Channel** installed and **elevated**.
- The TV and your computer on the same network, and **root SSH** that works from your computer
  (key-based is easiest). Try: `ssh -tt root@TV_IP 'echo ok'`.
- On your computer: `ssh`, `scp` and `sh`. Nothing else is needed to install a prebuilt package.
- Optional but recommended: turn **Quick Start+** on in the TV settings. It makes "off" a real
  standby and the TV wakes straight into Luma instead of cold-booting.

> [!NOTE]
> Always use `ssh -tt` (a terminal) when running `luna-send` on the TV. Without a terminal it
> returns empty output with a success status on webOS 6.5. The scripts already do this.

## 2. Get the package

Download `cc.ncls.luma_<version>_arm.ipk` and `sha256sums` from the
[Releases](https://github.com/nicolaseliasx/lg-hush/releases) page into the same folder, then verify:

```sh
sha256sum -c sha256sums --ignore-missing
```

(or build it yourself, see [Build](../readme.md#build)).

## 3. Install the app

```sh
scripts/install-tv.sh TV_IP path/to/cc.ncls.luma_<version>_arm.ipk
```

Add `--dry-run` to see every command without running any. The script copies the package to
`/tmp`, installs it with `appInstallService`, then runs the read-only diagnosis.

<details>
<summary>The same steps by hand</summary>

```sh
scp -O cc.ncls.luma_<version>_arm.ipk root@TV_IP:/tmp/lg-hush.ipk
ssh -tt root@TV_IP
```

In the TV's shell:

```sh
luna-send -i luna://com.webos.appInstallService/dev/install \
  '{"id":"cc.ncls.luma","ipkUrl":"/tmp/lg-hush.ipk","subscribe":true}'
```

Wait for the success message, then Ctrl+C. Open **Luma** from the TV's app list.

</details>

## 4. Check your TV (changes nothing)

The installer already runs this at the end. To run it again:

```sh
scripts/diagnose-tv.sh TV_IP
```

It prints the webOS release and whether the Home replacement is `possible` or `blocked`, with
the reasons. `possible` does not prove it works on your TV; it only means nothing the helper can
read blocks it.

## 5. Make it your Home (optional)

Supported methods exist for webOS 5, 6, 25 and 26. The takeover does **not** delete LG's Home: it
sits underneath a temporary mount that a reboot clears.

Disable any other Home replacement or Home-key remapping first (for example
[LG Input Hook](https://github.com/Simon34545/lginputhook)).

```sh
scripts/install-tv.sh TV_IP path/to/cc.ncls.luma_<version>_arm.ipk --home
```

or on the TV:

```sh
APP=/media/cryptofs/apps/usr/palm/applications/cc.ncls.luma   # Homebrew Channel install
# APP=/media/developer/apps/usr/palm/applications/cc.ncls.luma # developer-mode install
sh "$APP/home.sh" enable
```

`enable` shows what it will change and asks `y/n`. Afterwards, check by hand: the Home key from
another app, standby wake, and one **full cold boot**. Status and log:

```sh
sh /var/lib/ncls-luma-home/home.sh status
cat /var/lib/ncls-luma-home/boot.log
```

## 6. Debloat (optional)

Only works while Luma is the Home:

```sh
scripts/install-tv.sh TV_IP path/to/cc.ncls.luma_<version>_arm.ipk --home --tune
# or on the TV:
"$APP/luma-helper" tune apply
"$APP/luma-helper" tune status
```

What it does: switches off the LG preloads and services listed in `tv-helper/src/tune.rs`
(RAM-only mounts and a saved copy of LG's preload list), and adds LG ad hosts to a RAM copy of
`/etc/hosts`. Nothing is uninstalled and no system partition is written. The lists were built for
a `55UP751C0SF` on webOS 6.5.3; on other firmware check them first (see
[development.md](../development.md#performance-debloat-luma-helper-tune)).

Undo: `"$APP/luma-helper" tune restore`, or disable the Home replacement (which lifts it first).

## 7. Continue Watching and Stremio (optional)

The **Continue Watching** row shows the titles in progress in your Stremio library. Sign in once:

```sh
scripts/stremio-login.sh TV_IP            # asks for your Stremio e-mail and password
scripts/stremio-login.sh TV_IP --logout   # remove the saved key
```

The password goes to `api.strem.io` only; the TV keeps just the session key, readable by Luma.
Switch the row off in **Settings → Widgets → Continue Watching**. A card opens the official Stremio
app and shows the title to search for. Opening the title directly needs a Stremio build that
accepts launch parameters installed as `cc.ncls.stremiocards`, which is a separate third-party
project and is not distributed here.

## 8. Undo everything

```sh
scripts/uninstall-tv.sh TV_IP                # LG's Home returns; the app stays installed
scripts/uninstall-tv.sh TV_IP --remove-app   # and remove the app
```

Always disable the Home replacement **before** removing the app. If something goes wrong, reboot
the TV: without the boot hook, LG's Home comes back.

## Troubleshooting

| Symptom                                  | What to check                                                                                                                                                        |
| ---------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `luna-send` prints nothing               | You forgot `ssh -tt`.                                                                                                                                                |
| `cannot log in as root@…`                | TV not rooted, wrong IP, or your SSH key is not authorised on the TV.                                                                                                |
| LG's Home shows for ~15 s at boot        | Expected on webOS 6.5: the hook applies after LG's Home starts.                                                                                                      |
| Luma does not come back from an app      | `sh /var/lib/ncls-luma-home/home.sh status` and the boot log; then `disable`.                                                                                        |
| The TV reboots on its own around standby | Seen twice on the author's TV with Quick Start+ on, also from the remote's power button; cause not found, and turning Quick Start+ off has not been tested as a fix. |
| A debug port is open on the LAN          | The LG web-app inspector (port 9998) is open on the LAN of the author's TV. It is an LG service, lg-hush does not open it, and it has not been closed.               |
