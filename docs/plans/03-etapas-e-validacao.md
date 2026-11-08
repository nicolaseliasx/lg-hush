# Etapas e validação

Atualizado em 2026-11-07. As etapas 1 e 2 foram concluídas no desktop e
commitadas no fork. O primeiro incremento e o ajuste do diagnóstico da etapa
3 também estão commitados. A TV foi rooteada e o Luma como Home foi testado
no aparelho; as etapas de debloat seguintes continuam futuras e não autorizam
novas alterações na TV.

## Etapa 0 — Preparação concluída

- Criar apenas o fork do Luma, cloná-lo e configurar `origin` e `upstream`.
- Registrar a revisão original e escrever os planos com índice e referências.
- Conferir o vínculo do fork, remotes, identidade Git e preservação do checkout.
- Não criar commits ou pushes, instalar dependências ou alterar o código.

Aceite: estrutura acessível, planos escritos e árvore de trabalho do Luma
limpa e correspondente à revisão inicial documentada.

## Etapa 1 — Reproduzir o Luma original

Concluída no desktop em 2026-11-06. Resultados e limites em
[Baseline do Luma](./05-baseline-do-luma.md).

Com o início da etapa autorizado, as dependências foram preparadas em
contêineres e os testes, preview e empacotamento foram reproduzidos antes de
modificar a base. A configuração do sistema não foi alterada.

A inspeção inicial do devhub encontrou Node `20.19.2`, Git, gh e Xvfb, mas não
encontrou Rust/Cargo, Nix, pnpm ou pkg-config no PATH. O Luma exige ferramentas
adicionais e Node 22 ou superior; seu guia oferece um ambiente Nix com versões
fixadas. Na etapa 0, nenhuma dessas ferramentas foi instalada; na etapa 1,
foram usadas dentro de contêineres, sem instalação global no devhub.

Usar [AGENTS.md](../../AGENTS.md),
[development.md](../../development.md) e os scripts existentes como fontes
dos comandos. Registrar falhas da base antes de atribuí-las ao fork.

Aceite: baseline reproduzível, ferramentas documentadas, preview desktop e
pacote ARM validados, com limites de verificação explícitos.

## Etapa 2 — Identidade e ajustes visuais mínimos

**Identidade: concluída no desktop em 2026-11-06.** O fork usa
`cc.ncls.luma`, com estado, hooks e preferências separados (`ncls-*`). Os
scripts de Home e boot recusam rodar se houver integração do Luma upstream, e
as atualizações estão desligadas até o fork ter feed próprio. Validado:
format, check, lint, clippy, testes Rust e Node, quatro smokes desktop e
pacote ARM (`cc.ncls.luma_2.16.2_arm.ipk`).
Referências a `com.ecklf.luma` que restam são intencionais (impedir
desinstalar ou tratar o upstream como app comum). Na conclusão dessa etapa no
desktop, nada havia sido testado na TV; os testes posteriores estão abaixo.

**Visual mínimo: concluído no desktop em 2026-11-07.** Decisão
do usuário: fundo preto total e nada na Home além de abrir e fechar apps.
Um novo wallpaper `onyx` (preto puro) virou o padrão e o gradiente vem
desligado, porque o gradiente deixava o fundo azulado (máx. RGB 24/25/31; sem
ele, 0/0/0). Clima, saudação e data vêm desligados por padrão; o clima só liga
com `show_weather: true` explícito. HDMI, favoritos e o botão de sair do app
foram mantidos. Os testes de UI e os smokes que exercitam widgets e galeria
agora fixam o visual antigo explicitamente (`Preferences::legacy_defaults`
nos testes Rust; preferências semeadas nos smokes, com `onyx` oculto para
manter a ordem da galeria), e há testes dos novos padrões. Validado:
format, check, lint, clippy, testes Rust e Node, quatro smokes e pacote ARM
(SHA-256 `a037572285e6df4b387a61279ea6c64db732acda7846ef40cc6f1932963baeaf`).
A aparência real do preto não foi medida no painel.

Identidade e visual mínimo foram commitados juntos em `ea3cf66` na branch
`fork/identity-and-black-defaults` e publicados no fork (`origin`).

**Aviso de atualizações desligadas: commitado em `8ebd21d` em 2026-11-07.**
O card de Configurações → Sistema já não oferecia "Check again"
(a linha só aparece com atualização disponível); agora também diz, nas 13
línguas, que as atualizações estão desligadas nesta versão. A descrição do
Homebrew deixou de prometer atualizações, e um teste falha se ela voltar a
prometê-las enquanto as verificações públicas estiverem desligadas. O smoke
principal confere o texto na tela.

Priorizar opções existentes para o visual preto e cinza e a apresentação de
apps e HDMI. Preservar navegação por setas, OK, Back, ponteiro e rolagem. Não
portar a UI do Free My TV ou adicionar novos widgets nesta etapa.

Aceite: comportamento existente preservado, navegação exercitada no preview
e identidade/atualização separadas antes de instalar o fork na TV.

## Etapa 3 — Diagnóstico de privacidade e debloat

Estender o helper Rust existente com operações limitadas de diagnóstico,
prévia, aplicação e restauração. Investigar as interfaces reais antes de
fechar o contrato; não acrescentar execução arbitrária de comandos à UI.

Manter chamadas de sistema e rede fora da thread de renderização. Usar a
interface de sistema já existente para testar sem TV e sem alterar o host.

O primeiro incremento deve permitir consultar versão, privilégios e recursos
suportados e mostrar as ações possíveis sem aplicá-las. Nenhum perfil deve
ser considerado suportado apenas porque os serviços estão presentes.

**Primeiro incremento: concluído no desktop em 2026-11-07.** O comando
`luma-helper diagnose` lê `/etc/starfish-release` e verifica root,
`luna-send`, `mount`/`umount`, `init.d` e restos da integração do Luma
upstream. Responde com JSON e com a ação `home-replacement` como `possible`
ou `blocked`, com os motivos, e `verified: false` sempre. Não escreve nem
executa comandos (testado comparando a árvore de arquivos antes e depois).
Cobre versão desconhecida, versão fora de 5/6/10/11, ausência de root,
ferramentas ausentes e conflito com o upstream. `possible` só significa que
nada encontrado bloqueia; não prova compatibilidade. Ainda não aparece na UI.
Na conclusão inicial, a leitura real de `/etc/starfish-release` e das
ferramentas ainda precisava ser confirmada no aparelho. O pacote ARM compila
(SHA-256 `1524f5b0da49efd327962aeecbc9f10b8b15b388e7aa7771af8c6bc8dc0024f6`).
Commitado em `cef3174` na mesma branch.

**Ajuste do diagnóstico: commitado em `8ebd21d` em 2026-11-07.** O
veredito deixou de usar uma lista própria de versões e passou a seguir as
mesmas verificações somente leitura do `enable`, na mesma ordem: o critério de
`home::method` (no webOS 5/6, manifest do Home `native`, `main`
`file:///usr/bin/com.webos.app.home` e `nativeLifeCycleInterfaceVersion` 2),
montagens alheias sobre o diretório ou o executável do Home, hook
`64-ncls-luma-home` de outro dono, hook `65-ncls-luma` e manifest que não é o
do Home da LG. O JSON informa o método (`executable` ou `manifest`) e perdeu
o campo `release.known`; firmware fora de 5/6/10/11 agora mostra a mensagem do
`enable`. Continua somente leitura e com `verified: false`. Formato e
significado documentados em
[development.md](../../development.md#read-only-helper-diagnosis). O pacote
`cc.ncls.luma_2.16.2_arm.ipk` dessa revisão foi instalado na TV, como
registrado abaixo.

Aceite: versão desconhecida, ausência de root e capacidade indisponível
produzem diagnóstico claro e nenhuma alteração.

## Etapa 4 — Primeiro controle reversível

Adiada: a prioridade atual do usuário é manter o Luma estável como Home.

Com modelo e firmware conhecidos, definir uma lista explícita de ações
suportadas. Começar por configurações de privacidade e bloqueio local de
destinos específicos de anúncios/telemetria, com IPv4 e IPv6 considerados.
Não importar integralmente blocklists ou scripts dos candidatos.

Antes da primeira alteração, salvar o estado original por ação e preservar
essa primeira cópia nas reaplicações. Verificar o resultado, informar falhas
parciais e permitir repetir restauração sem apagar modificações de terceiros.

Detectar conflitos de hosts e bind mounts; recusar aplicação conflitante é
preferível a coordenar gerenciadores independentes. A restauração deve poder
ser acionada sem depender da interface do launcher.

Aceite: testes de falha parcial e restauração passam; no aparelho, a ação
altera apenas o recurso previsto e volta ao estado anterior.

## Etapa 5 — Serviços, preloads e persistência

Adiada pelo mesmo motivo; não confundir o hook já ativo da Home com um hook de
debloat.

Somente após inventário e restauração comprovada, investigar serviços e
preloads individualmente. Preservar DRM, streaming, configurações, áudio,
rede e HDMI. Ocultar apps na interface não implica bloquear sua execução ou
remover arquivos do sistema.

Preloads de apps usados não devem ser desativados automaticamente. Comparar
o ganho de RAM com a perda no tempo de abertura. Não criar uma lista genérica
de serviços seguros baseada apenas nos nomes usados por outro firmware.

Persistência no boot entra após validar aplicação e restauração manual, com
um caminho para desabilitar o hook e recuperar o Home original. O inventário
da TV é necessário para fechar esse perfil; não há perfil definitivo agora.

## Estratégia de testes para o desenvolvimento

Para funcionalidades e correções, escrever primeiro um teste de regressão,
confirmar sua falha, implementar a menor mudança e executar novamente os
testes e verificações relevantes.

Cobrir pelo menos:

- Sem root, firmware desconhecido, configuração ausente e serviço indisponível.
- Aplicação repetida preservando o primeiro estado original.
- Falha no meio de uma ação, nova tentativa e restauração parcial ou completa.
- Restauração preservando entradas e mudanças alheias.
- IPv4/IPv6 e conflitos de gerenciamento de hosts.
- Navegação e acesso a apps/HDMI após ajustes visuais.

Usar as suítes existentes de Rust e ferramentas, smoke tests desktop, lint,
verificação de formatação e build ARM/IPK conforme as alterações futuras.
Para mudanças apenas na documentação local, não repetir essas suítes.

## Validação no aparelho e desempenho

Antes de ativar controles, validar o Luma como app e depois a substituição do
Home, incluindo restauração. Conferir Home/Back, standby, reinício, áudio,
apps utilizados, reprodução DRM e HDMI/CEC quando utilizado.

Registrar medições antes e depois de cada grupo de mudanças, repetindo os
mesmos cenários de abertura fria e quente: latência da Home, navegação,
abertura de apps e CPU/RAM em condições equivalentes. Relatar regressões e
compensações, sem usar o consumo de RAM isoladamente como prova de velocidade.

Para privacidade, relacionar cada destino e serviço ao resultado observado.
Ausência de tráfego num intervalo não comprova eliminação de toda coleta.
Preview, mocks e CI não comprovam compatibilidade ou desempenho na TV.

## Primeiro contato com a TV (2026-11-07)

Autorizado pelo usuário, com o Developer Mode ligado e o Key Server ligado
só durante a obtenção da chave. A chave decifrada e o perfil do `ares` ficaram
num diretório temporário isolado e foram apagados; nada foi gravado na home do
usuário. O Key Server deve voltar a ser desligado na TV.

- `ares-device -i`: `55UP751C0SF`, SDK `6.5.3`, firmware `03.53.45`, placa
  `LM21A_ATSC_BR`.
- O acesso é o usuário `prisoner` do Developer Mode, em jail: sem root, sem
  `/etc/starfish-release` e sem `/var/lib/webosbrew`. Não há Homebrew Channel.
- Instalado o pacote `cc.ncls.luma_2.16.2_arm.ipk` (commit `8ebd21d`,
  SHA-256 `c198d63017419b22052954b8c627dfa2f81083e8568dd0b250a369cc18bf9da8`)
  como app de desenvolvimento, e lançado; o `ares-launch --running` confirmou
  que ele abriu. Nenhuma integração de Home ou hook foi ativada.
- `luma-helper diagnose` rodou na TV e respondeu: `root: false`, release
  desconhecido (jail), `luna-send` e `mount` presentes, `initD: false`, sem
  integração upstream, e `home-replacement` bloqueado com os motivos "Run the
  TV helper as root" e "Unknown webOS version". O comando funciona no
  aparelho, somente leitura, e o resultado confirma que a substituição do Home
  e qualquer controle que exija root ficam bloqueados até haver root.
- Sem verificação: a aparência real do fundo preto e a navegação com o
  controle, que dependem do usuário olhar a TV.

## Root da TV com SlopBro (2026-11-07)

Autorizado pelo usuário após a apresentação do requisito de remover o app
Developer Mode da LG antes do primeiro reinício. Foi usada a revisão
`d7e6d4e4653a1d997c55b1e131be1cd255122a6f` do
[SlopBro](https://github.com/throwaway96/slopbro), sem alterar os payloads, com
`--webos-version 6 --asset-source dir`. Python 3.13.5 atendeu ao requisito
3.8+ do projeto; os 24 testes locais do SlopBro passaram. Relatos de sucesso
para o mesmo firmware e webOS existiam em outros modelos, mas não confirmavam
este modelo específico.

- A TV aceitou o pareamento SSAP e baixou os quatro arquivos do payload. A foto
  do usuário mostrou `Rooting complete` e o aviso para remover Developer Mode.
- O app Developer Mode da LG foi removido antes do primeiro reinício. O
  Homebrew Channel 0.7.3 apareceu após reiniciar; o Luma continuou listado.
- No acesso root temporário, `id -u` retornou `0`. A chave pública SSH local
  existente foi instalada em `/home/root/.ssh/authorized_keys`, sem copiar a
  chave privada para a TV. O Homebrew Channel foi configurado com SSH ligado e
  Telnet desligado.
- Após outro reinício, SSH autenticado exclusivamente por chave retornou UID
  `0`, o hook `/var/lib/webosbrew/startup.sh` estava presente, Telnet estava
  fechado, e a lista de apps ainda continha Homebrew Channel e Luma, sem
  Developer Mode.
- `luma-helper diagnose` rodou somente leitura por SSH root e retornou release
  `Rockhopper release 6.5.3-47 (kisscurl-koli)`, `root: true`, `initD: true`,
  `upstreamIntegration: false` e `home-replacement: possible` com
  `verified: false`. A integração com a Home não foi ativada nem testada.

Sem verificação: navegação visual do Luma, apps de streaming, HDMI, DRM e
compatibilidade da substituição da Home ou do debloat. Nenhum controle de
privacidade ou hook próprio foi aplicado.

## Teste do Luma como Home (2026-11-07)

Autorizado pelo usuário com restauração preparada. Antes da ativação,
`luma-helper home status` mostrou `enabled: false`, `mounted: false` e
`bootHook: false`; `home preflight` retornou sucesso. Os scripts instalados
estavam presentes, e não havia outros hooks em `/var/lib/webosbrew/init.d`.
Nesta TV, consultas `luna-send` por SSH sem PTY terminavam com status zero e
stdout vazio; `ssh -tt` retornou JSON válido para foreground, lista de 112
apps e inventário de entradas. HDMI 1 e 2 tinham `hdmiPlugIn: false`, de
acordo com a informação do usuário de que não há equipamento conectado.

- `home.sh enable` foi executado por SSH com PTY e confirmação explícita. O
  helper relatou `enabled: true`, método `executable`, alvo
  `com.webos.app.home`, montagem e hook `64-ncls-luma-home` ativos.
- O usuário abriu YouTube, Stremio e Spotify pelo Luma e confirmou que a tecla
  Home voltou ao Luma. Relatou reprodução funcional de áudio/vídeo quando
  aplicável.
- `home.sh disable` retornou sucesso. O `umount` comum encontrou o executável
  ocupado e o script fez desmontagem lazy; o status posterior mostrou
  `enabled: false`, `mounted: false`, `bootHook: false`, sem marcador. O usuário
  confirmou que a tecla Home voltou à Home da LG.
- A integração foi reativada. Após desligar e ligar pelo controle, a tecla
  Home abriu o Luma. Após desligar, tirar da tomada por cerca de 30 segundos e
  ligar, Home voltou a abrir o Luma. Stremio, YouTube e Spotify continuaram
  visíveis, e pelo menos um app abriu e voltou com Home. SSH por chave ainda
  retornou UID 0, e o helper mostrou integração, montagem e hook ativos após
  a partida fria.

O campo `homeKeyVerified` do status permanece `null` por projeto; a verificação
da tecla Home acima foi manual. Há um único ciclo de espera e de partida fria
testado. HDMI, CEC/ARC, uso prolongado e debloat não foram testados. O comando
de restauração permanece `sh /var/lib/ncls-luma-home/home.sh disable` por SSH
com PTY.

[Índice](./README.md)
