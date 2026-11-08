# Baseline do Luma original

Validado em 2026-11-06 no commit `1e9b7d6e4956573fee9558b55ff4911d6265bf02`,
versão `2.16.2`, antes de qualquer mudança no código do fork.

## Ambiente e resultados

- Dependências instaladas com `pnpm install --frozen-lockfile`.
- Flake do Luma executado no contêiner `nixos/nix:2.35.4`: Node 24.21.0,
  pnpm 10.34.5, Rust 1.93.0 e pkg-config 0.29.2.
- `pnpm format:check`, `pnpm check`, `pnpm test` e `pnpm assets` passaram.
- Os quatro scripts de smoke desktop passaram com Node 24, Rust 1.93,
  SDL2, Mesa e Xvfb em um contêiner Debian, seguindo as dependências da CI:
  `tests/smoke.mjs`, `tests/brands-smoke.mjs`, `tests/weather-smoke.mjs` e
  `tests/translations-smoke.mjs`. O smoke principal cobriu navegação,
  renderização e persistência; a galeria renderizou 40 apps demonstrativos.
- `pnpm sdk`, `pnpm runtime` e `pnpm run package` passaram no contêiner Nix,
  com o carregador e bibliotecas do host montados somente para leitura para
  executar o compilador do SDK webOS.
- IPK gerado e verificado: `../../.build/baseline/com.ecklf.luma_2.16.2_arm.ipk`,
  SHA-256 `c8d8dc0adb84ec976393cff0e0bf5177002f9ec733af9e9b5691289692f55846`.
  O pacote ainda usa a identidade original `com.ecklf.luma`.

O Xvfb dentro do contêiner Nix não forneceu um contexto GLX compatível;
por isso o smoke desktop foi executado no contêiner Debian, como na CI do
upstream. Esse problema do ambiente Nix não foi atribuído ao código do Luma.
Não houve instalação ou execução na TV. Preview e pacote desktop não provam
compatibilidade do aparelho, root ou substituição do Home.

## Reprodução no devhub

Execute os comandos a seguir a partir da raiz do repositório. O flake é copiado com seu
lockfile para dentro do contêiner porque o Git recusa que o root do contêiner
leia o checkout pertencente ao usuário do host. As montagens de `/lib64` e
`/lib/x86_64-linux-gnu` permitem executar o compilador do SDK webOS, sem
alterar bibliotecas do host.

```sh
docker run --rm -v webos-lab-nix-store:/nix -v "$PWD:/work" \
  -v /lib64:/lib64:ro -v /lib/x86_64-linux-gnu:/lib/x86_64-linux-gnu:ro \
  -w /work -e 'NIX_CONFIG=experimental-features = nix-command flakes' \
  nixos/nix:2.35.4 sh -ec '
    mkdir -p /tmp/luma-flake
    cp /work/flake.nix /work/flake.lock /work/package.json /tmp/luma-flake/
    nix develop path:/tmp/luma-flake --command sh -ec "
      pnpm install --frozen-lockfile --store-dir .build/pnpm-store
      pnpm format:check
      pnpm check
      pnpm test
      pnpm assets
      pnpm sdk
      pnpm runtime
      export WEBOS_NDK=/work/.build/toolchain/arm-webos-linux-gnueabi_sdk-buildroot
      pnpm run package
    "'
```

O smoke desktop foi reproduzido com uma imagem equivalente às dependências
gráficas instaladas pela CI do Luma. Construa a imagem local:

```sh
docker build -t webos-lab-smoke:baseline - <<'DOCKERFILE'
FROM node:24-bookworm AS node
FROM rust:1.93-bookworm
COPY --from=node /usr/local/bin/node /usr/local/bin/node
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
      libsdl2-dev pkg-config libgl1-mesa-dri xvfb xauth \
    && rm -rf /var/lib/apt/lists/*
DOCKERFILE
```

```sh
docker run --rm -v "$PWD:/work" -v webos-lab-debian-cargo:/usr/local/cargo \
  -w /work -e CARGO_TARGET_DIR=/work/.build/debian-target \
  -e LIBGL_ALWAYS_SOFTWARE=1 -e LUMA_SMOKE_SKIP_BUILD=1 \
  webos-lab-smoke:baseline sh -ec '
    cargo build --locked --package luma
    xvfb-run -a -s "-screen 0 1920x1080x24" sh -ec "
      node tests/smoke.mjs
      node tests/brands-smoke.mjs
      node tests/weather-smoke.mjs
      node tests/translations-smoke.mjs
    "'
```

Os contêineres criam arquivos com dono `root` no bind mount. Após a execução,
devolva os artefatos gerados ao usuário do host sem mudar os arquivos do Git:

```sh
docker run --rm -v "$PWD:/work" -w /work nixos/nix:2.35.4 \
  sh -ec "chown -R $(id -u):$(id -g) .build target dist node_modules"
```

## Próxima etapa

Atualizado em 2026-11-07. A identidade separada e o visual mínimo já estão no
fork (`ea3cf66`), e o diagnóstico somente leitura (`cef3174`) foi ajustado para
seguir os critérios do `enable` (sem commit). Os passos seguintes são:

1. Revisar e commitar, com autorização, as mudanças pendentes no clone.
2. Confirmar modelo, versão interna do webOS e root (ver
   [Dados pendentes da TV](./04-dados-pendentes-da-tv.md)).
3. Com autorização específica para a TV, instalar o pacote do fork e rodar
   `luma-helper diagnose` como root. O firmware informado sugere webOS 6; o
   resultado mostra se o manifest do Home da LG permite o método `executable`.
4. Só depois validar o Luma como app e a substituição do Home com
   restauração, conforme [Etapas e validação](./03-etapas-e-validacao.md).

Para essas mudanças no clone, os testes foram rodados sem rede
(`--network none`): `pnpm format:check`, `pnpm check` e `pnpm test` no
contêiner Nix, montando também os volumes `webos-lab-nix-cache` (em
`/root/.cache/nix`) e `webos-lab-cargo-cache` (em `/root/.cargo`), sem os quais
o Nix e o Cargo tentam baixar; e os smokes no contêiner Debian acima. Os
artefatos de build ficam em diretórios ignorados pelo Git; o acesso de
desenvolvimento fica em `../../.env` local.

[Índice](./README.md)
