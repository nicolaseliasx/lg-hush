# Planos do projeto (antes “webOS Lab”)

Atualizado em 2026-11-07. O baseline do Luma original foi validado no
desktop e a separação de identidade do fork (`cc.ncls.luma`, atualizações
desligadas) e o visual mínimo (fundo preto `onyx`, sem clima, saudação e
data) foram implementados, validados no desktop e commitados no fork. O
diagnóstico somente leitura do helper (etapa 3, 1º incremento) e o ajuste que
o alinha aos critérios do `enable` estão commitados. A TV foi rooteada com
SlopBro; o Luma passou nos testes de Home, restauração, espera e partida fria
descritos em [03](./03-etapas-e-validacao.md). A prioridade atual do usuário é
estabilidade do Luma. Controles de debloat não foram iniciados.

| Documento                                                                | Finalidade                                                                       |
| ------------------------------------------------------------------------ | -------------------------------------------------------------------------------- |
| [01 — Objetivos e direção](./01-objetivos-e-direcao.md)                  | Registrar o resultado desejado, as escolhas confirmadas e os limites.            |
| [02 — Referências de debloat](./02-referencias-de-debloat.md)            | Comparar projetos e identificar ideias a investigar para nosso debloater.        |
| [03 — Etapas e validação](./03-etapas-e-validacao.md)                    | Organizar o desenvolvimento futuro e seus critérios de aceite.                   |
| [04 — Dados pendentes da TV](./04-dados-pendentes-da-tv.md)              | Reunir as informações necessárias antes de definir o perfil do aparelho.         |
| [05 — Baseline do Luma](./05-baseline-do-luma.md)                        | Registrar os testes desktop e o pacote ARM da base original.                     |
| [06 — Relato da inicialização](./06-relato-inicializacao-do-luma.md)     | Registrar que a Home da LG aparece antes do Luma ao ligar a TV.                  |
| [07 — Plano de otimização do início](./07-plano-otimizacao-do-inicio.md) | Quick Start+, boot frio mais curto, Home sem cursor e debloat de desempenho.     |
| [08 — Seção "Para Você"](./08-plano-secao-para-voce.md)                  | Fileira de recomendações de filmes e séries na Home, com fonte, layout e aceite. |

## Como ler o estado do projeto

- **Preparado e validado no desktop:** fork, clone, documentação, testes do
  Luma original, smoke de renderização e pacote ARM.
- **Confirmado no aparelho:** root por SSH com chave, Homebrew Channel, Luma
  como Home, retorno pela tecla Home de YouTube/Stremio/Spotify, restauração da
  Home da LG, retomada da espera e partida fria.
- **Escopo atual:** manter o Luma estável para os apps usados; nenhuma fonte
  HDMI está conectada. Debloat e controles de privacidade ficam para depois.
- **Pendente de evidência:** uso prolongado, outras funções da TV e qualquer
  ganho de desempenho.

Esta etapa não estabelece uma lista definitiva de serviços a desativar. Isso
depende do modelo, firmware, inventário e testes de restauração no aparelho.

[Voltar ao projeto](../../readme.md)
