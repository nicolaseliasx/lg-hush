# Relato: a Home da LG aparece antes do Luma

Data: 2026-11-07. TV: LG `55UP751C0SF`, webOS `6.5.3`.

Ao ligar a TV, aparece primeiro a interface inicial padrão da LG webOS.
Depois de um intervalo que o usuário estima em cerca de 10 segundos, o Luma
abre sozinho. A partir daí, ele funciona como o usuário espera: os apps
aparecem e a tecla Home volta ao Luma após abrir um app.

Este arquivo registra apenas o comportamento observado. Nenhuma mudança foi
feita na TV para tratar essa inicialização.

## Diagnóstico somente leitura (2026-11-07)

Lido por SSH root no boot frio das 13:42 (horário local), sem alterar a TV.
Tempos em segundos desde o boot do kernel (`/var/log/messages`):

| Tempo | Evento                                                                                                                              |
| ----: | ----------------------------------------------------------------------------------------------------------------------------------- |
|  20,7 | `bootManager` abre a Home da LG (`NL_HOME_SHOWN bootingHome`); logo some em 22,5.                                                   |
|   ~35 | `startup.sh` do Homebrew Channel começa (disparado por `devmode.conf`).                                                             |
|   ~43 | Após `sleep 2` do startup, `run-parts` e `sleep 5` do `home.sh boot`, o `apply` monta o Luma e encerra a Home da LG (some em 43,4). |
| 43–60 | Nenhuma janela de Home visível; o SAM não registra lançamento pedido pelo helper.                                                   |
|  60,0 | `preloadmanager` relança `com.webos.app.home`, agora o binário do Luma.                                                             |
|  61,4 | Luma visível.                                                                                                                       |

Quick Start+ (`quickStartMode`) está `off`: cada vez que a TV liga é um boot
frio. `homeAutoLaunch` está `on`. O upstream já documenta que a Home da LG pode
aparecer até o hook aplicar. Hipótese não confirmada para o intervalo 43–60:
o `launch` do helper foi aceito, mas não gerou lançamento, e a leitura do app
em primeiro plano ainda indicava `com.webos.app.home`, encerrando a espera.

Também observado: `preloadmanager` pré-carrega livetv, HDMI 2, Netflix e
YouTube no boot; o bloqueio de atualizações do Homebrew Channel está
desligado; nenhum controle de debloat foi aplicado. Neste acesso, `luna-send`
sem PTY retornou JSON.

[Índice](./README.md)
