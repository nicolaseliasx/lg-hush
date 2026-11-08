# Dados pendentes da TV

Atualização de 2026-11-07: modelo, SDK e firmware foram confirmados no aparelho.
O SlopBro concluiu o root; após dois reinícios, SSH com chave retornou UID 0,
o Homebrew Channel apareceu, e Telnet permaneceu fechado. O diagnóstico leu
`Rockhopper release 6.5.3-47 (kisscurl-koli)` como root. A versão de software
`03.53.45` foi informada pelo usuário.

## Informações necessárias

| Informação                                         | Estado                                                                                                                   | Uso                                                                            |
| -------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------ |
| Modelo completo, com sufixo regional               | `55UP751C0SF`, placa `LM21A_ATSC_BR`, lido pelo `ares-device -i` em 2026-11-07                                           | Identificar hardware e relatos de compatibilidade.                             |
| Versão do webOS                                    | SDK `6.5.3` pelo `ares-device -i`; `/etc/starfish-release` lido após root: `Rockhopper release 6.5.3-47 (kisscurl-koli)` | Identificar a versão interna efetiva.                                          |
| Versão de software/firmware                        | `03.53.45`, informada pelo usuário                                                                                       | Conferir root, Home e operações suportadas após identificar o modelo.          |
| Root e Homebrew Channel existentes                 | Root confirmado por SSH com chave após reinício; Homebrew Channel 0.7.3 instalado                                        | Acesso root disponível para diagnósticos e futuras etapas autorizadas.         |
| Developer Mode e rede                              | App Developer Mode da LG removido antes do primeiro reinício; TV na mesma rede, SSH com chave ativo e Telnet desligado   | Evitar reinstalar Developer Mode enquanto a TV estiver rooteada.               |
| Apps utilizados                                    | Stremio, YouTube e Spotify; abriram e voltaram ao Luma pela tecla Home                                                   | Preservar abertura, reprodução e retorno ao launcher.                          |
| Equipamentos HDMI e áudio utilizados               | Nenhum equipamento HDMI conectado; saída de áudio específica não informada                                               | Entradas sem cabo ficam ocultas no modo Auto; HDMI/CEC/ARC não foram testados. |
| Outros launchers, hooks ou bloqueadores existentes | Nenhum outro hook em `/var/lib/webosbrew/init.d` antes do teste; o hook `64-ncls-luma-home` agora está ativo             | Evitar conflitos futuros de Home, serviços e hosts.                            |

## Preferências já confirmadas

- Root é aceito e faz parte da direção desejada.
- A interface inicial será a do Luma com poucos ajustes, em tons escuros.
- Stremio, YouTube e Spotify são os casos de uso prioritários agora; não há
  equipamento HDMI conectado.
- A prioridade atual é manter o Luma estável como Home. O debloater no fork é
  direção futura; não instalar um conjunto de soluções nesta fase.

Voz, casting e integrações LG não foram definidos como requisitos. Isso não
autoriza desligá-los silenciosamente: se alguma ação futura afetá-los, seu
impacto deve ser apresentado antes de aplicá-la.

## Decisões que dependem desses dados

- Perfil inicial de configurações, destinos e serviços de debloat.
- Cenários de medição e reprodução dos apps e equipamentos realmente usados.

Consultar as instruções específicas do método vigente e a documentação do
[webOS Homebrew](https://www.webosbrew.org/rooting/) quando a versão exata
for conhecida. Guias e matrizes podem estar defasados; registrar fontes e
datas, sem tratar exemplos antigos como confirmação do firmware atual.

O firmware `03.53.45` aparece em [vários modelos relatados pelo
CleanYourTV](https://github.com/lasharor/cleanyourtv-WebOS/blob/dev/compat.json)
com SDK 6.5.3. Para obter o modelo sem compartilhar número de série, a
[LG Brasil orienta pressionar Mute três vezes no
controle](https://www.lg.com/br/suporte/suporte-ao-producto/solucionar-problemas/help-library/cs-CT52001752-1434560282225OLT/).

Até esses dados chegarem, manter aplicação de controles de debloat como
trabalho futuro. A integração com a Home e seu hook foram testados e estão
ativos; sua restauração foi verificada. Não há necessidade de coletar
senhas, tokens de streaming ou dados de conta para preencher este documento.
O IP e a antiga passphrase de desenvolvimento recebidos em foto ficam apenas
no arquivo local `.env`, ignorado pelo Git; não os copiar para
documentos ou commits. A chave pública SSH foi instalada na TV; a chave
privada não foi copiada para ela.

[Índice](./README.md)
