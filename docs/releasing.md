# Cutting a release

Releases are made by hand. Upstream's `release-pr` and `changeset-status` workflows were removed
because they publish a Homebrew-style feed with upstream's settings; `ci.yml` and `pr-test-ipk.yml`
remain.

1. **Validate** on the pinned toolchain: `pnpm format:check`, `pnpm check`, `pnpm test`, `pnpm assets`,
   `pnpm test:smoke` (the main smoke takes several minutes and must run on an otherwise idle machine).
2. **Version.** Every release-worthy change has a file in `.changeset/` (`pnpm changeset`). Run
   `pnpm version:release`: it consumes them, bumps `package.json`, `appinfo.json`, `Cargo.toml`,
   `Cargo.lock` and the IPK names in `readme.md`, and rewrites `CHANGELOG.md`. Check with
   `pnpm version:check`, then commit.
3. **Build** the package from the committed tree: `scripts/build-ipk-docker.sh`. It writes
   `dist/cc.ncls.luma_<version>_arm.ipk`, `sha256sums`, `repo.json`, `description.html` and
   `app-icon-large.png`.
4. **Check the package** before publishing: `sha256sum -c dist/sha256sums`, and `ar t` on the IPK
   lists `debian-binary`, `control.tar.gz`, `data.tar.gz`. Install it on a TV with
   `scripts/install-tv.sh` and open Luma. A desktop build never proves TV behaviour.
5. **Tag and publish:**

   ```sh
   git tag -a v<version> -m "lg-hush <version>"
   git push origin main v<version>
   gh release create v<version> --title "lg-hush <version>" --notes-file <notes.md> \
     dist/cc.ncls.luma_<version>_arm.ipk dist/sha256sums dist/repo.json \
     dist/description.html dist/app-icon-large.png
   ```

`dist/repo.json` is a Homebrew Channel feed whose URLs point at this repository's release assets
(`tools/repository.mjs`). Nothing publishes it as a feed automatically; add it to Homebrew Channel
by hand only if you want in-app updates, and keep update checks off otherwise.
