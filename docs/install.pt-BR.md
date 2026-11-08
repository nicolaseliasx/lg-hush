# Instalando o lg-hush numa TV webOS

[English](install.md) · [Voltar ao README](../readme.md)

> [!WARNING]
> É preciso uma TV **com root**. Este guia não cobre o root; siga o
> [guia de root do webosbrew](https://www.webosbrew.org/rooting/) para o seu modelo e firmware. Tudo é
> **por sua conta e risco** e foi testado em uma única TV (veja [Tested on](../readme.md#tested-on)).
> Os scripts de instalação não foram executados em outros modelos.

## 1. Pré-requisitos

- TV LG webOS com root, **Homebrew Channel** instalado e **elevado**.
- TV e computador na mesma rede, com **SSH como root** funcionando (por chave é mais simples).
  Teste: `ssh -tt root@IP_DA_TV 'echo ok'`.
- No computador: `ssh`, `scp` e `sh`. Para instalar um pacote pronto não precisa de mais nada.
- Opcional, recomendado: ligue o **Quick Start+** nas configurações da TV. Assim "desligar" vira
  standby de verdade e a TV volta direto no Luma, sem boot frio.

> [!NOTE]
> Use sempre `ssh -tt` (com terminal) ao rodar `luna-send` na TV. Sem terminal ele devolve saída
> vazia com status de sucesso no webOS 6.5. Os scripts já fazem isso.

## 2. Baixe o pacote

Baixe `cc.ncls.luma_<versão>_arm.ipk` e `sha256sums` da página de
[Releases](https://github.com/nicolaseliasx/lg-hush/releases) para a mesma pasta e verifique:

```sh
sha256sum -c sha256sums --ignore-missing
```

(ou compile você mesmo: [Build](../readme.md#build)).

## 3. Instale o app

```sh
scripts/install-tv.sh IP_DA_TV caminho/cc.ncls.luma_<versão>_arm.ipk
```

`--dry-run` mostra todos os comandos sem executar nenhum. O script copia o pacote para `/tmp`,
instala com o `appInstallService` e roda o diagnóstico somente leitura.

<details>
<summary>Os mesmos passos à mão</summary>

```sh
scp -O cc.ncls.luma_<versão>_arm.ipk root@IP_DA_TV:/tmp/lg-hush.ipk
ssh -tt root@IP_DA_TV
```

No shell da TV:

```sh
luna-send -i luna://com.webos.appInstallService/dev/install \
  '{"id":"cc.ncls.luma","ipkUrl":"/tmp/lg-hush.ipk","subscribe":true}'
```

Espere a mensagem de sucesso e dê Ctrl+C. Abra o **Luma** na lista de apps.

</details>

## 4. Verifique a TV (não altera nada)

```sh
scripts/diagnose-tv.sh IP_DA_TV
```

Mostra a versão do webOS e se a troca da Home está `possible` ou `blocked`, com os motivos.
`possible` não prova que funciona na sua TV: só indica que nada que o helper consegue ler a bloqueia.

## 5. Usar como Home (opcional)

Há métodos para webOS 5, 6, 25 e 26. A troca **não apaga** a Home da LG: ela fica por baixo de um
mount temporário que o reboot limpa. Desative antes qualquer outra troca de Home ou remapeamento
da tecla Home (por exemplo o [LG Input Hook](https://github.com/Simon34545/lginputhook)).

```sh
scripts/install-tv.sh IP_DA_TV caminho/cc.ncls.luma_<versão>_arm.ipk --home
```

ou na TV:

```sh
APP=/media/cryptofs/apps/usr/palm/applications/cc.ncls.luma   # instalado pelo Homebrew Channel
# APP=/media/developer/apps/usr/palm/applications/cc.ncls.luma # instalado em modo desenvolvedor
sh "$APP/home.sh" enable
```

O `enable` mostra o que vai mudar e pergunta `y/n`. Depois confira à mão: tecla Home vindo de
outro app, volta do standby e um **boot frio completo**. Estado e log:

```sh
sh /var/lib/ncls-luma-home/home.sh status
cat /var/lib/ncls-luma-home/boot.log
```

## 6. Debloat (opcional)

Só funciona enquanto o Luma é a Home:

```sh
scripts/install-tv.sh IP_DA_TV caminho/cc.ncls.luma_<versão>_arm.ipk --home --tune
# ou na TV:
"$APP/luma-helper" tune apply
"$APP/luma-helper" tune status
```

Desliga os preloads e serviços da LG listados em `tv-helper/src/tune.rs` (mounts só em RAM e uma
cópia da lista de preloads da LG) e adiciona hosts de anúncios da LG a uma cópia em RAM do
`/etc/hosts`. Nada é desinstalado e nenhuma partição do sistema é escrita. As listas foram feitas
numa `55UP751C0SF` com webOS 6.5.3; em outro firmware, confira antes
([development.md](../development.md#performance-debloat-luma-helper-tune)).

Desfazer: `"$APP/luma-helper" tune restore`, ou desativar a troca da Home (ela desfaz o debloat antes).

## 7. Linha "For You" e Stremio (opcional)

Ligue **For You** nas configurações do Luma; ela só aparece com saudação, data e clima
desligados. Um card abre o Stremio oficial e mostra o título para você buscar. Abrir o título
direto exige um build do Stremio que aceite parâmetros de lançamento instalado como
`cc.ncls.stremiocards`, um projeto de terceiros à parte que não é distribuído aqui.

## 8. Desfazer tudo

```sh
scripts/uninstall-tv.sh IP_DA_TV                # a Home da LG volta; o app continua instalado
scripts/uninstall-tv.sh IP_DA_TV --remove-app   # e remove o app
```

Desative a troca da Home **antes** de remover o app. Se algo der errado, reinicie a TV: sem o
hook de boot, a Home da LG volta.

## Problemas comuns

| Sintoma                                   | O que verificar                                                                                                                                                        |
| ----------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `luna-send` não imprime nada              | Faltou `ssh -tt`.                                                                                                                                                      |
| `cannot log in as root@…`                 | TV sem root, IP errado ou chave SSH não autorizada na TV.                                                                                                              |
| A Home da LG aparece por ~15 s no boot    | Esperado no webOS 6.5: o hook aplica depois que a Home da LG inicia.                                                                                                   |
| O Luma não volta de um app                | `home.sh status` e o log de boot; depois `disable`.                                                                                                                    |
| A TV reinicia sozinha em torno do standby | Visto duas vezes na TV do autor com Quick Start+ ligado, também pelo botão do controle; causa não encontrada, e desligar o Quick Start+ não foi testado como correção. |
| Porta de debug aberta na rede local       | O inspetor de apps web da LG (porta 9998) está aberto na rede da TV do autor. É um serviço da LG, o lg-hush não o abre e ele não foi fechado.                          |
