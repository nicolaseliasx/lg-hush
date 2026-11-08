# Plano: seção "Para Você" na Home

Data: 2026-11-07. Pedido do usuário: uma fileira de recomendações de filmes e
séries entre a barra superior e "Favorites", ocupando o espaço vazio com o
mesmo estilo escuro. Referência visual: card de destaque largo seguido de
cards menores, com título e "gênero · duração · origem" abaixo. Nada foi
implementado; este plano aguarda aprovação.

## O que a TV permite (verificado)

- Nenhum app de streaming (Netflix, Prime, Disney+, Max, Apple TV,
  Globoplay) expõe catálogo ou recomendações a outro app no webOS. A única
  agregação existente é a da própria LG (fileira "Tendências agora" da Home
  da LG), que depende do `contentminer` e de servidores da LG, desligados no
  debloat por decisão do usuário. "Puxar dos apps em Favorites" não tem fonte
  local.
- O app de filmes e séries realmente usado é o Stremio. O catálogo público do
  Stremio (Cinemeta, sem chave nem conta) respondeu em 2026-11-07:
  `catalog/movie/top.json` (49 títulos, ~124 KB) e `catalog/series/top.json`
  (50, ~559 KB), todos com `name`, `genres`, `runtime`, `releaseInfo`,
  `imdbRating` e id IMDb. Imagens em `images.metahub.space`:
  `background/small` 480×270 JPEG (~26 KB) e `background/medium` 1920×1080
  (~780 KB). Títulos e gêneros vêm em inglês.
- O Luma já tem HTTP com TLS (`ureq` + `rustls`), decodificação PNG/JPEG/WebP
  (`image`) e o padrão de worker com cache e modo offline do clima
  (`weather-service.rs`), que serve de modelo.

## Decisões propostas

1. **Fonte: Cinemeta.** Sem chave, combina com o Stremio. Alternativa para
   depois: TMDB com provedores do Brasil (dados do JustWatch), que diz em qual
   serviço cada título está, mas exige chave de API do usuário e só abre o app
   do serviço, sem ir ao título.
2. **"Inteligente" na v1 = curadoria simples e honesta:** intercalar filmes e
   séries dos catálogos "top" e girar a seleção a cada atualização do cache.
   Não há histórico de visualização acessível; não fingir personalização.
   Origem exibida: "Stremio".
3. **Ação do OK:** abrir o Stremio na tela inicial e mostrar um aviso no Luma
   com o nome do título, para o usuário buscar. Testado na TV em 2026-11-07
   (Stremio `io.strem.tv` 1.13.3, abertura do zero, conferido pelo usuário):
   o manifesto declara `deeplinkingParams: {"contentTarget":"stremio"}`, mas
   `contentTarget` com `stremio:///detail/movie/tt0111161`,
   `stremio:///detail/movie/tt0111161/tt0111161`,
   `https://web.stremio.com/#/detail/movie/tt0111161/tt0111161` e
   `stremio:///search?search=The%20Shawshank%20Redemption` abriu sempre na
   tela inicial. O código do app é cifrado no `cryptofs` e não há
   documentação pública do parâmetro. Se surgir um formato que funcione, a
   troca fica restrita ao payload de lançamento.
4. **Layout:** fileira "Para Você" no topo (card de destaque 16:9 maior + cards
   16:9 menores), Favorites deslocada para baixo com aparência e comportamento
   idênticos. O espaço livre atual (y ≈ 245–470 em 1920×1080) não comporta o
   destaque sem mover Favorites.
5. **Opção em Configurações** ("Mostrar Para Você"), ligada após a aprovação.
   Desligada, a Home fica exatamente como hoje (decisão anterior: Home preta e
   mínima).

## Arquitetura

- `src/for_you.rs`: modelo (`Item { id, kind, title, genres, runtime, year,
  rating, image }`), parser do Cinemeta, tradução de gêneros (tabela fixa nas
  13 línguas), seleção de até 12 itens intercalados.
- `src/for-you-service.rs`: worker HTTP fora da thread do SDL, resposta
  limitada a 1 MiB, timeout curto; cache em disco no armazenamento persistente
  (JSON e imagens) com validade de 6 h; offline mostra o cache. Imagens
  baixadas uma a uma em `background/small` (o destaque em `medium`,
  reduzido a 960×540 ao decodificar). Primeira busca só depois do primeiro
  quadro e com a Home ociosa.
- Estado: itens, texturas, `Target::ForYou(i)`, rolagem própria da fileira
  (mesmo padrão de `home_scroll`).
- Vista: título "Para Você" (23,2 px, peso 550, como "Favorites"), cards com
  raio 19,2 px (o dos tiles), foco com o mesmo destaque dos tiles, legenda em
  duas linhas: título; "gênero · duração · Stremio". Esqueleto enquanto
  carrega; seção oculta se não houver dados nem cache.
- Navegação: setas Cima/Baixo entre "Para Você" e Favorites, Esquerda/Direita
  na fileira, OK abre, ponteiro com hover e clique, como os tiles.

## Orçamento de desempenho

Texturas: 11 cards 480×270 RGBA (~5,7 MB) + destaque 960×540 (~2 MB). Sem
efeito no tempo até o Luma aparecer (cache local desenhado no primeiro quadro
ou seção vazia). Medir RAM e swap na TV antes e depois; reverter se a
memória disponível cair mais que ~15 MB além das texturas.

## Passos verificáveis (TDD)

1. Spike na TV, somente leitura e um lançamento: testar parâmetro de
   lançamento do Stremio para abrir um título. Resultado decide a ação do OK.
2. Testes do parser com respostas gravadas do Cinemeta, gêneros traduzidos,
   duração ausente, campos inválidos, limite de tamanho.
3. Testes do serviço: cache válido, expirado, offline, resposta maior que o
   limite, nenhuma rede na thread do SDL (seguir os testes do clima).
4. Testes de estado: foco entre fileiras, rolagem, OK gera `Effect::Launch`,
   opção desligada mantém a Home atual intacta.
5. Vista e smoke desktop com servidor de teste local (como o smoke do
   clima): seção renderizada, Favorites idêntica com a opção desligada.
6. Pacote ARM, instalação e medição na TV: boot frio, RAM, swap, navegação
   pelo controle.

## Critérios de aceite

- Com a opção desligada, a Home é idêntica à atual (smoke compara).
- Com a opção ligada: até 12 cards com imagem, título, gênero e duração;
  destaque à esquerda; navegação por setas e ponteiro; OK abre o Stremio.
- Sem rede: cache ou seção oculta, sem travar a Home.
- Boot frio sem regressão (Luma visível ~43 s) e memória dentro do
  orçamento.
- Format, lint, clippy, testes Rust/Node, quatro smokes e pacote ARM passam.

## Estado em 2026-11-07 (noite)

Decisões finais do usuário: título "For You" (sem "Luma"), cards 16:9 do
mesmo tamanho, sem abas, OK abrindo direto no título por um segundo Stremio,
"Stremio Cards", mantendo o oficial.

**Stremio Cards** (projeto de terceiros mantido só localmente, fora deste repositório; clone de
`kieranbrown/stremio-webos` no commit `9a8f10a`, branch local `cards`, sem
licença declarada: uso pessoal, não publicar). Id `cc.ncls.stremiocards`,
título "Stremio Cards"; ffmpeg armhf (a TV é `armv7l`; o upstream empacota
arm64), SHA-256 `7d41f558…3991`, MD5 igual ao publicado pelo autor; deep link
`{type, imdbId}` aplicado quando o frontend chega em `#/home` (ele trocava o
hash inicial) e também com o app aberto (`webOSRelaunch`); reaproveita o
servidor de streaming do oficial quando a porta 11470 está ocupada e sobe o
próprio se o proxy receber ECONNREFUSED; `inspectable: false`; log em
`/tmp/stremiocards-service.log`. Testes Node: 9. Validado na TV pelo usuário:
deep link frio e com o app aberto, reprodução com o servidor do próprio Cards
(dono da 11470 conferido), legendas e áudio pt/en, avançar e voltar. A porta
9998 (inspetor do gerenciador de apps web da LG) segue aberta na LAN; não é do
Cards.

**Luma**: seção "For You" no espaço livre acima de Favorites, que não se move;
aparece só com saudação, data e clima desligados. Detalhes técnicos em
`luma/development.md` ("For You" row). Testes novos: dados (5), serviço com
servidor HTTP local (3, inclui redirecionamento e rejeição de não-JPEG),
estado e tela (6). Suíte: Luma 310, helper 71, Node, lint e clippy limpos.
Capturas desktop com dados reais conferidas (cards, legenda em português,
foco). Pendente: smokes e pacote desta versão, instalação e medição na TV.

### Ajustes de design e uso (2026-11-07, madrugada)

- Pôsteres 2:3 (`poster/small`, 300×450, WebP convertido para JPEG no worker)
  no tamanho nativo, sem legenda por card; só o título em foco numa linha
  abaixo da fileira. Favoritos desce 245 px enquanto a fileira aparece. Cache
  movido para `preferences.for-you-posters*` e o 16:9 antigo é apagado.
- Stremio Cards pré-carregado pelo Luma (`preload: "full"`): 10 s após a
  primeira tela e depois a cada 30 s com a Home visível (Voltar dentro dele o
  fecha). Na TV: janela em ~2 s vindo do pré-carregamento, 0,04 s já aberto.
  Custa ~175 MB. Fica fora do painel de apps abertos.
- Painel de apps abertos com cards em escala 0,6, centralizados.
- Relógio de 24 h ligado nas preferências da TV (`hour_24`).

Validado: Luma 314, helper 71, Node, lint, clippy e quatro smokes; pacote
SHA-256 `5bd61226…f7cdd491a43227cbbcfaac` instalado na TV. Sem commit (aguardando
aprovação do usuário); Stremio Cards nunca deve ser publicado.

### Desligar em vez do protetor de tela (2026-11-08)

Pedido do usuário: em vez do relógio de inatividade da LG, desligar de fato
(Quick Start+ ligado). Nesta TV o protetor é um estado do `tvpower`
(`getPowerState` = `"Screen Saver"`), não um app em primeiro plano. Como
Home, o Luma lê esse estado a cada 2 s e chama `power/powerOff`
(`reason: "remoteKey"`) assim que ele aparece (`src/sleep.rs`, 2 testes).
Validado na TV: `Screen Saver → Suspend` em ~5 s e retorno `Suspend → Active`
sem reiniciar.

Tempo até o protetor: nesta TV LCD o padrão é `screenSaverTimeout` 1800 s
(`com.webos.service.nop`, `/usr/sbin/nopd`); os 120 s/30 s do `configd` valem
só para OLED. O valor pode ser mudado com `configd setConfigs`
(`com.webos.service.nop.screenSaverTimeout`), mas o `nopd` só o lê ao iniciar:
é preciso `systemctl restart nopd`. Testado em 2026-11-08 com 120 s (protetor 2
min após o restart, desligado 5 s depois) e deixado em 1800 s a pedido do
usuário. A contagem recomeça a cada tecla do controle (o `nopd` registra
`removeNopTimerHandler … (option:reset)`). Conferido com 1800 s: última tecla
às 00:08:57 e protetor às 00:38:59, seguido do desligamento.

Observado e não resolvido: duas vezes a TV reiniciou por conta própria em
torno do Quick Start (`Init → Active Standby` com `PowerOnReason: quickStart`
depois de um desligamento, e `Active → Reboot` com `PowerOffReason:
quickStart` 21 s depois de ligar pelo controle). Ocorreu também com o botão do
controle, então não vem do Luma; a causa não foi capturada (o webOS limpa
`/var/log` no boot).

[Índice](./README.md)
