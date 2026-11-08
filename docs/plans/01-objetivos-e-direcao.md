# Objetivos e direção

Estado: Luma em uso como Home; debloat adiado. Atualizado em 2026-11-07.

## Resultado desejado

Ter uma TV LG webOS com interface simples, bonita e predominantemente preta,
orientada aos apps usados. A prioridade atual é a estabilidade do Luma como
Home. Reduzir anúncios, componentes indesejados e coleta de dados permanece
como direção futura, condicionada a medições e testes no aparelho.

## Escolhas confirmadas

- Usar o [Luma](https://github.com/ecklf/lg-luma) como base do fork, mantendo
  Rust, SDL2 e OpenGL ES e fazendo poucas mudanças no visual inicial.
- Construir nosso debloater dentro do fork. Estudar outros projetos para
  aproveitar ideias, mecanismos documentados e, quando adequado, código com
  suas licenças e avisos preservados.
- Criar somente o fork do Luma nesta fase. Os
  [pretendentes de referência](./02-referencias-de-debloat.md) não são outros
  forks nem dependências já escolhidas.
- Preservar apps de streaming, reprodução com DRM, HDMI, controle remoto e
  configurações de imagem e som.
- Root foi confirmado na `55UP751C0SF`. O Luma como Home passou nos testes
  iniciais de apps, restauração, espera e partida fria.

## Primeira experiência visual pretendida

Manter a composição atual do Luma e aproveitar suas opções existentes antes
de propor novas funções. Priorizar fundo preto, tons de cinza, foco legível,
favoritos e acesso às entradas quando houver equipamento conectado. Não adicionar notícias, recomendações, clima
ou widgets à primeira experiência pretendida.

O Free My TV apareceu como possível complemento, mas a pesquisa encontrou
um launcher na versão webOS, sem mecanismo de debloat de serviços. Seu
visual pode inspirar melhorias futuras; não há decisão de portar sua UI.

## Direção técnica a investigar

O Luma já tem um [helper Rust](../../tv-helper/src/lib.rs) para operações
privilegiadas e uma [interface de sistema](../../tv-helper/src/system.rs)
que permite testes com substitutos. A primeira hipótese é estender essa
estrutura, mantendo trabalho de sistema fora da renderização.

Uma futura área de configurações deve permitir entender o efeito de uma ação,
consultar seu estado e restaurá-la. O contrato detalhado desses controles
será definido na etapa de desenvolvimento, com base nos recursos suportados.

Separar três objetivos: ocultar um ícone muda a apresentação; impedir a
execução de um componente altera o sistema; bloquear um destino de rede
reduz comunicação com aquele destino. Um não comprova os outros.

## Restrições e critérios de sucesso

- Presença de um serviço não comprova que seja seguro desativá-lo.
- Versão desconhecida deve ficar em diagnóstico, sem aplicação automática.
- Cada alteração precisa de estado original salvo e restauração verificável.
- Não alterar proteções OLED, firmware, DRM, rede ou HDMI como tentativa
  genérica de acelerar a TV.
- Reduzir preloads pode liberar RAM e aumentar o tempo de abertura de apps;
  a escolha deve considerar o uso real e medições.
- Medir Home, navegação e abertura de apps; Rust ou menos processos não são
  evidência suficiente de melhoria.
- Descrever os destinos e controles efetivamente verificados, sem prometer
  ausência de toda coleta de dados.

Modelo e firmware estão em [04](./04-dados-pendentes-da-tv.md). A substituição
da Home foi testada separadamente da capacidade de abrir o Luma como app;
uso prolongado e funções não exercitadas continuam sem validação. Ver a
[documentação original](../../readme.md) e as
[pendências do aparelho](./04-dados-pendentes-da-tv.md).

[Índice](./README.md)
