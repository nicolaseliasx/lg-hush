# Referências e candidatos para debloat

Consulta: 2026-11-06. Valores de estrelas são um retrato da consulta à API do
GitHub, não um ranking de segurança, maturidade ou desempenho. A pesquisa
consultou documentação, trechos de código, licenças e registros de CI; nenhum
dos controles foi executado na TV.

## Base escolhida

[Luma — ecklf/lg-luma](https://github.com/ecklf/lg-luma): 327 estrelas,
criado em 2026-10-19, último push consultado em 2026-11-02, licença MIT.
Revisão clonada: `1e9b7d6e4956573fee9558b55ff4911d6265bf02`, versão `2.16.2`.

O launcher usa Rust/SDL2/OpenGL ES e tem preview desktop, testes e um helper
privilegiado. O suporte à substituição do Home varia conforme webOS e modelo;
não inferir compatibilidade de uma TV recente sem identificar o firmware.
Esses mecanismos ainda não representam um debloater do sistema.

Fontes: [README](../../readme.md),
[desenvolvimento](../../development.md),
[helper](../../tv-helper/src/lib.rs).

## Pretendentes

| Projeto                                                             | Estrelas | Último push consultado | Papel proposto                                                               |
| ------------------------------------------------------------------- | -------: | ---------------------- | ---------------------------------------------------------------------------- |
| [Glasshouse](https://github.com/rorygallagher2024/glasshouse)       |    1.921 | 2026-11-06             | Referência principal para privacidade, anúncios, serviços e preloads.        |
| [CleanYourTV WebOS](https://github.com/lasharor/cleanyourtv-WebOS)  |      434 | 2026-11-06             | Referência para ações por grupos, integração com Home e reversão.            |
| [Own Your Glass](https://github.com/JulioFerrero/own-your-glass)    |      439 | 2026-10-27             | Investigações de coleta e bloqueios; estudar com atenção às restrições.      |
| [LG TV Blocklist](https://github.com/furkan-bayrak/lg-tv-blocklist) |      314 | 2026-10-30             | Referência de destinos de anúncios e telemetria, com níveis de bloqueio.     |
| [Free My TV](https://github.com/simbasjungles/free-my-tv)           |    5.955 | 2026-10-31             | Referência secundária de experiência visual, sem debloat webOS identificado. |

### Glasshouse

Oferece controles de privacidade, gestão de apps e serviços, preloads e
bloqueio local de anúncios e telemetria. Tem testes e CI e também oferece
uma Home alternativa, que não precisamos incorporar para estudar os controles.

Investigar a separação entre configurações LG, bloqueios por hosts e serviços;
a detecção de capacidades por versão; e a preservação de valores anteriores.
Seu backend em JavaScript não se integra automaticamente ao renderer Rust.

O dashboard fica sem autenticação por padrão. Instalá-lo separadamente
introduziria esse aspecto operacional, mas essa instalação não faz parte do
plano atual. Sua compatibilidade geral não comprova a do launcher ou de cada
ação individual.

Licença principal: MIT. Fontes:
[visão geral](https://github.com/rorygallagher2024/glasshouse),
[privacidade](https://github.com/rorygallagher2024/glasshouse/blob/main/server/lib/privacy.js),
[apps e background](https://rorygallagher2024.github.io/glasshouse/dashboard/apps/),
[testes](https://rorygallagher2024.github.io/glasshouse/development/testing/).

### CleanYourTV WebOS

Organiza ações para anúncios, ACR/LivePlus, logs e sugestões. Documenta
substituições temporárias em memória, desligamento por grupo e restauração.
Também descreve controles de preload e suas diferenças entre firmwares.

Estudar o ciclo aplicar/verificar/restaurar e como evita alterações
permanentes nos binários originais. O debloat documentado depende de o
launcher estar controlando o Home; não copiar seus scripts presumindo que
sejam independentes desse contexto.

O bloqueio básico documentado cobre poucos destinos, sem eliminar toda coleta
nem bloquear firmware por si só. Seu suporte declarado começa no webOS 5.

A API do GitHub classificou a licença como `NOASSERTION`; o arquivo principal
consultado contém MIT e aviso OFL de fonte. Há componentes de terceiros com
licenças próprias, inclusive GPL; revisar os arquivos efetivamente usados
antes de copiar código.

Fontes: [debloat](https://lasharor.github.io/cleanyourtv-WebOS/debloat/),
[implementação](https://github.com/lasharor/cleanyourtv-WebOS/blob/dev/helper/debloat.js),
[licença](https://github.com/lasharor/cleanyourtv-WebOS/blob/dev/LICENSE).

### Own Your Glass

Documenta módulos de ACR, voz, captura, logs, serviços e rede, com diagnóstico,
aplicação e restauração. É útil para descobrir hipóteses e falhas já encontradas.

O próprio README restringe o firmware testado a webOS 10/webOS 25 e relata
problemas graves com uma versão anterior no webOS 4, inclusive alterações
que não foram revertidas por restauração de fábrica. Não usar como receita
genérica nem forçar execução em firmware desconhecido.

Investigar seus achados e os módulos individualmente; não importar o conjunto
de ações padrão ou seus rótulos de segurança sem comprovação na nossa TV.
Licença principal: MIT.

Fonte: [README e limitações](https://github.com/JulioFerrero/own-your-glass).

### LG TV Blocklist

Lista destinos de anúncios e telemetria e distingue níveis de bloqueio.
Pode ajudar a relacionar um domínio com sua finalidade e possíveis efeitos
colaterais. A licença consultada é CC-BY-4.0, com exigência de atribuição.

Uma lista para DNS precisa de adaptação se usada em hosts: hosts combina
nomes exatos, sem bloquear automaticamente todos os subdomínios. Rever também
IPv4, IPv6, destinos compartilhados com loja e login e evidências regionais.
Não importar listas grandes indiscriminadamente.

Fonte: [repositório](https://github.com/furkan-bayrak/lg-tv-blocklist).

### Free My TV

É o mais popular entre os launchers encontrados na pesquisa. Na documentação
consultada, a edição LG instala um app via Developer Mode ou Homebrew Channel
e mantém o Home original. Não foi identificado um mecanismo de debloat de
serviços webOS para trazer ao Luma.

Pode inspirar experiência de uso; a UI LG é HTML/CSS/JavaScript, enquanto a
do Luma é nativa. Portá-la exigiria implementação adicional, fora da direção
de poucos ajustes visuais escolhida agora.

Os números de 60 fps e redução de RAM destacados no README foram medidos em
uma TCL Google TV com Android 14. Não servem como benchmark de LG webOS.
Licença principal: MIT.

Fontes: [edição LG](https://github.com/simbasjungles/free-my-tv#lg-tvs-webos),
[interface web](https://github.com/simbasjungles/free-my-tv/tree/main/web).

## Como aproveitar as referências

Antes de adotar uma ideia, registrar sua fonte, o firmware observado, o efeito
pretendido, dependências, possíveis impactos, evidência de aplicação e método
de reversão. Confirmar no código e no aparelho; alegações do README permanecem
alegações do projeto até validação própria.

A intenção é construir o debloater no Luma. Não instalar vários gerenciadores
de Home, serviços ou hosts ao mesmo tempo. Se outro gerenciador controlar um
recurso, identificar o conflito antes de qualquer aplicação.

[Índice](./README.md)
