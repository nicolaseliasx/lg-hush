# Plano: início rápido, cursor e debloat de desempenho

Data: 2026-11-07. Prioridade definida pelo usuário: otimização do início e da
fluidez acima de qualquer outro trabalho. O usuário autorizou executar o que
não exige sua ajuda nem desligar a TV; o restante aguarda nova permissão.

## Evidência de partida

- Linha do tempo do boot frio em [06](./06-relato-inicializacao-do-luma.md):
  Home da LG visível de ~21 s a 43 s, nenhuma Home de 43 s a 60 s, Luma em 61 s.
- Quick Start+ `off`: cada vez que a TV liga é um boot frio.
- Memória: 1,3 GB total, ~257 MB disponíveis e ~465 MB em swap com o Luma
  como Home. A pressão de memória é um custo real de fluidez.
- Política de pré-carregamento (`getPreloadPolicy`), com `isAllowed: true`:
  YouTube (`semi-full`, 135 MB), Netflix, navegador, voz, livemenu, livetv,
  HDMI 2 e a Home (`criu`). Itens com `isAllowed: false` já não são
  pré-carregados nesta TV.
- Em execução: `contentminer`, `nudge`, `sportsalarm`, `admanager`,
  `remotelogger`, `rdxd`, `uploadd`. `acr2` e `adoverlay-service` existem, mas
  não estavam rodando na leitura.
- Bloqueio de atualizações do Homebrew Channel desligado
  (`webosbrew_block_updates` ausente).

## Fase A — Quick Start+ (configuração da TV)

Ligar `quickStartMode`. Desligar pelo controle passa a ser standby, e ligar
volta direto ao último estado, já com o Luma como Home, sem o boot.
Compensação: consumo maior em standby. A montagem e o processo do Luma ficam
em RAM e devem sobreviver; isso não foi testado. Boot frio continua
acontecendo após falta de energia, tirar da tomada ou reinício da LG, por isso
as fases B e D seguem necessárias.

Reversão: desligar a opção no menu ou por `setSystemSettings`.

Aceite: desligar e ligar pelo controle três vezes; o Luma aparece sem a Home
da LG; tecla Home e um app continuam funcionando; `home status` segue ativo.

## Fase B — Boot frio mais curto (código, TDD)

1. **Confirmar que o Luma abriu de fato.** Hoje `open_executable` aceita o
   app em primeiro plano `com.webos.app.home` como sucesso, mas logo após o
   encerramento da Home da LG esse valor pode ser antigo. Teste de regressão
   no `System` falso do helper: primeiro plano informa a Home, mas nenhum
   processo do Luma existe; o helper deve relançar até o processo do Luma
   aparecer ou esgotar o prazo. Ganho esperado: até ~17 s.
2. **Remover o `sleep 5` do `home.sh boot`.** Quando o hook do Homebrew roda
   (~35 s), o SAM já abriu a Home há ~15 s. Trocar por aplicação imediata,
   mantendo o processo destacado. Atualizar `tests/home-wrapper.test.mjs`
   primeiro. Ganho esperado: ~5 s.

Limite: a Home da LG continua visível de ~21 s até o hook rodar (~36 s). Agir
antes exigiria alterar a partição do sistema, fora de escopo.

Aceite: testes Rust e Node, lint, clippy e smokes passam; pacote ARM gerado;
na TV, um boot frio medido pelos logs com Luma visível antes de ~40 s.

## Fase C — Home sem cursor (código, TDD)

Objetivo: a tecla Home sempre leva à aba inicial em modo de foco por setas, sem
o cursor do Magic Remote; o cursor só volta ao chacoalhar ou mover o controle,
como o sistema já faz.

O Luma já zera o modo ponteiro em `return_home`, mas o cursor é desenhado pelo
sistema. A SDL do webOS exporta `SDL_webOSCursorVisibility` (símbolo presente
em `libSDL2-2.0.so.0.3000.12`). Chamar `SDL_webOSCursorVisibility(false)` no
relançamento como Home (`code == 1`) e na tecla Home. Teste de regressão: os
dois caminhos produzem o efeito de esconder o cursor; caminhos de ponteiro não.

Risco: o sistema pode mostrar o cursor de novo logo após o foco. Se isso
acontecer, repetir a chamada no `FocusGained` seguinte ao relançamento.

Aceite na TV: com o cursor visível num app, Home leva ao Luma sem cursor e com
o foco destacado; ao mover o controle, o cursor reaparece e funciona.

## Fase D — Debloat de desempenho

Comando novo no helper, `luma-helper tune apply|status|restore`, com hook
próprio (`66-ncls-luma-tune`) separado do hook da Home. Nada é desinstalado
nem gravado nas partições do sistema. Antes da primeira aplicação, salva o
estado original por ação e não o sobrescreve em reaplicações. Restauração por
SSH sem depender da interface. Testes de aplicação repetida, falha parcial e
restauração antes de tocar na TV.

1. **Pré-carregamentos** (método do Glasshouse): `setPreloadPolicy` no boot e
   ao sair do standby, mais `configd setConfigs` persistente. Desligar
   Netflix e navegador. YouTube e entradas dependem da decisão abaixo. Não
   fechar apps já carregados; só impedir o próximo carregamento.
2. **Serviços de anúncio, sugestões e telemetria** (método do CleanYourTV,
   stub em RAM montado sobre o binário, só enquanto o Luma é Home):
   `contentminer`, `nudge`, `sportsalarm`, `adoverlay-service`, `acr2` e
   `remotelogger`. `admanager` só após verificar que nenhum app usado depende
   dele.
3. **Hosts**: bloquear os três destinos de anúncio documentados pelo
   CleanYourTV, em IPv4 e IPv6, numa cópia em RAM montada sobre `/etc/hosts`,
   coordenada com o bloqueio do Homebrew.
4. **Bloqueio de atualizações de firmware** (flag do Homebrew Channel):
   recomendado, porque uma atualização da LG pode remover o root.

Aplicar um grupo por vez, medindo antes e depois: momento em que o Luma fica
visível no boot, memória disponível e swap, e tempo de abertura fria de
YouTube, Stremio e Spotify. Reverter qualquer grupo que quebre reprodução,
áudio ou a tecla Home.

## Estado em 2026-11-07

Sem commit no clone (`fork/identity-and-black-defaults`).

- **A:** `quickStartMode` ligado por `setSystemSettings` e confirmado por
  leitura. Ciclo de standby ainda não testado (precisa desligar a TV).
- **B:** `open_executable` só aceita sucesso com um processo novo da Home; no
  boot relança a cada 4 s por até 25 s e, se esgotar, mantém a montagem em vez
  de desfazê-la (o lançamento do `preloadmanager` ainda abre o Luma). O
  `home.sh boot` não espera mais 5 s e registra o uptime em `boot.log`; o
  helper registra uptime de cada lançamento e da chegada à frente.
- **C:** `return_home` emite `Effect::HideCursor`, que chama
  `SDL_webOSCursorVisibility(false)` no build da TV. A SDL embutida exporta o
  símbolo.
- **D:** `luma-helper tune apply|restore|status`. Exige o Luma como Home;
  `home disable` desfaz o tune antes; o `apply` de boot reaplica um tune
  ativo. Pré-carregamentos desligados: Netflix e navegador. Serviços com stub:
  `contentminer`, `nudge`, `remotelogger`, `sportsalarm` (`/usr/bin`), `acr2`
  e `adoverlay-service`. Três hosts de anúncio em IPv4 e IPv6.

Validado no desktop: testes de regressão escritos antes e vistos falhando;
`pnpm format:check`, `pnpm check` (oxlint e clippy `-D warnings`) e `pnpm test`
(Luma 296, helper 63, suítes Node); quatro smokes; pacote ARM
`cc.ncls.luma_2.16.2_arm.ipk`, SHA-256
`594aec37c2197bfc5cba1eed581970e9693fbfddfcf0aa0917c1fabded3f82fd`.

Na TV: pacote instalado por `appInstallService/dev/install` e Home reaplicada
com `home.sh enable` (scripts copiados sem `sleep 5`; Luma à frente em 1,6 s).
Linha de base antes do tune: ~273 MB disponíveis, ~449 MB de swap usados.
`tune apply` **não foi executado** (permissão negada no ambiente do agente).

Pendente com o usuário: ciclo de standby (A), boot frio medido pelo
`boot.log` (B), cursor ao apertar Home e ao chacoalhar (C), `tune apply` com
medição de memória e abertura de YouTube, Stremio e Spotify (D), e as decisões
sobre YouTube pré-carregado, entradas/voz e bloqueio de firmware.

### Atualização após o primeiro boot frio (2026-11-07, tarde)

Boot frio com o pacote anterior: hook a 36,5 s, Home da LG fechada a 36,8 s
(antes 43,4 s), Luma visível a 59,4 s (antes 61,4 s). O `boot.log` mostrou
que o helper nem lançava o Luma: após encerrar a Home da LG o app em primeiro
plano fica vazio, e a checagem "ainda na Home" exigia o id da Home. Corrigido
(vazio conta como Home), com teste de regressão. O tune de boot falhou só no
`setPreloadPolicy` (preload manager ainda sem resposta a 36 s); agora tenta de
novo por até 20 s. A configuração persistente bastou para Netflix e navegador.
Após o boot: swap usado 100 MB, disponíveis ~476 MB.

Decisões do usuário: YouTube indiferente (mantido); sem antena, HDMI ou voz
(pré-carregamento desligado para TV aberta, livemenu, HDMI 1–4, Miracast,
voz, Amazon, Globoplay e Google Assistant; ficam Home, YouTube e AirPlay);
bloquear firmware da LG mantendo atualização de apps
(`webosbrew_block_updates` criado; linhas da LG adicionadas também à cópia em
RAM do hosts, porque boots frios ficam raros com Quick Start+).

Outras correções feitas no aparelho, todas com teste antes: `sportsalarm`
roda de `/usr/sbin` iniciado pelo barramento (stub e encerramento);
`systemctl stop --no-block` porque o `nudge` ignora SIGTERM além do prazo; o
stub não é regravado a cada `apply`, e a posse da montagem é lida em
`mountinfo` (incluindo `//deleted`).

Pacote instalado: SHA-256
`55816cd1269d730afd6d0257d5e30e4da47e9050fb35318d46fa7beff8bad7ce`; helper 67
testes, Luma 296, lint e clippy limpos. Testes do usuário: standby, cursor e
apps "quase tudo funcional". Pendente: novo boot frio para medir a correção do
lançamento.

### Segundo boot frio e rodada 2 (2026-11-07, noite)

Boot frio medido: Home da LG de 20,9 s a 38,5 s, hook a 38,2 s, lançamento a
39,4 s, Luma visível a 43,5 s (antes 61,4 s). O usuário viu ~14 s de Home da
LG e ~2 s de carregamento. O Luma desenha a primeira tela 0,9 s após iniciar;
o resto é a criação do processo pelo sistema durante o boot.

Rodada 2, pedida pelo usuário (não usa ThinQ, HomeKit nem AirPlay):
`admanager`, `rdxd`, `uploadd`, `homekit`, `airplay` e o serviço JS
`com.webos.service.homeconnect` mascarados e parados; AirPlay fora dos
pré-carregamentos; SIGKILL após 3 s para serviços que ignoram SIGTERM
(`homekit`); falha do `setPreloadPolicy` no boot só gera aviso. `lmt`
(limitar rastreamento de anúncios) ligado. Live Plus, promoção da Home e os
consentimentos de anúncio, ACR, voz e diagnóstico já estavam desligados.
Resultado: os 11 alvos sem processo; swap usado 47 MB, ~535 MB disponíveis.

Commitado e publicado no fork: `fc4ad8d` (cursor) e `7d5242b` (boot, tune e
documentação, incluindo a nota pendente do diagnóstico). Pacote na TV:
SHA-256 `6594296b84f344bed96252f26560627eb7b6eb2cd2afa9a4110deeaf87a1a94d`.

Pendente: observar alguns dias se, após o standby, nada volta a rodar e os
apps continuam normais. Não foi feito novo boot frio após a rodada 2.

### Rodada 3: coleta e envio (2026-11-07, noite)

Conexões TCP externas lidas de `/proc/net`: só `iconnectivity` (teste de
conexão da LG), YouTube e o SSH. Sem endereços de telemetria em texto nos
binários; o envio já está cortado na origem (`uploadd`, `rdxd`,
`remotelogger`), e não foram bloqueados domínios sem evidência, para não
afetar loja e login. `webos-statistics-manager` não existe neste firmware.
Mascarados e parados: adaptadores Alexa (2), Google Assistant, ThinQ AI e
Dangbei, e `broadcast-conf-downloader` (systemd, `Restart=on-failure`).
Commit `a6ad57b`; 15 alvos sem processo; swap 52 MB, ~555 MB disponíveis.

## Ordem de execução

A → B + C (um pacote) → D grupo a grupo. Cada etapa na TV pede confirmação.

[Índice](./README.md)
