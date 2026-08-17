# Portmaster API {{VERSION}}

Este pacote traz três coisas:

| Arquivo | O que é |
|---|---|
| `portmaster-api-http` | O binário. Estático, musl, `linux/amd64`. Não tem dependência de sistema. |
| `portmaster-api-image.tar` | A mesma aplicação como imagem de contêiner, para quem não roda Linux ou não quer o binário solto. |
| `README.md` | Este arquivo. |

As migrations **não** estão aqui. Elas são um artefato à parte
(`portmaster-migrations-<versão>.tar.zst`), porque quem aplica schema e quem sobe
a API costumam ser passos diferentes do mesmo deploy.

---

## Escolha rápida

- **Linux x86-64** — pode usar o binário direto. É o caminho mais leve: um
  processo, ~8 MB, sem runtime.
- **macOS, Windows, ou qualquer outra arquitetura** — use a imagem. O binário é
  compilado para `linux/amd64` e não roda nativamente em nenhum dos dois.
- **Servidor com systemd** — binário mais unit file, na seção correspondente.

---

## Configuração

A aplicação lê tudo do ambiente e **não sobe** se faltar algo obrigatório — é
deliberado: um servidor que sobe sem saber onde está o banco só adia a falha
para a primeira requisição.

| Variável | Obrigatória | O que é |
|---|---|---|
| `APP_DB_HOST` | sim | Host do MariaDB. |
| `APP_DB_PORT` | sim | Normalmente `3306`. |
| `APP_DB_NAME` | sim | O banco, já criado e migrado. |
| `APP_DB_USER` | sim | Usuário. |
| `APP_DB_PASSWORD` | sim | Senha. |
| `APP_JWT_SECRET` | sim | Segredo HS256. **Mínimo 32 bytes** — mais curto que o digest é recusado. |
| `APP_HOST` | não | Onde escutar; padrão `0.0.0.0`. |
| `APP_PORT` | não | Porta; padrão `8000`. |

O boot confere o banco antes de anunciar-se saudável. Credencial errada derruba o
processo ali, e não na primeira requisição.

### Provar que subiu

`GET /info` é a única rota pública com corpo, e responder nela prova mais do que
uma porta aberta: significa que o boot passou, o banco respondeu e o catálogo de
permissões foi preenchido.

```
curl -fsS http://localhost:8000/info
```

### O primeiro usuário

Uma instalação nova não tem ninguém, e as rotas protegidas não deixam criar o
primeiro. Quem resolve isso é `POST /setup`, que cria o usuário inicial junto de
um papel com todas as permissões registradas — e recusa rodar uma segunda vez.

```
curl -fsS -X POST http://localhost:8000/setup \
  -H 'Content-Type: application/json' \
  -d '{"name":"Admin","email":"admin@exemplo.com","password":"TroqueIsto1"}'
```

---

## Docker / Podman

Carregue a imagem, e ela passa a existir com o nome que o próprio arquivo
carrega:

```sh
docker load -i portmaster-api-image.tar
docker image ls | grep portmaster
```

`podman load -i portmaster-api-image.tar` funciona igual — é um arquivo OCI.

```sh
docker run -d --name portmaster-api \
  -p 8000:8000 \
  -e APP_DB_HOST=mariadb \
  -e APP_DB_PORT=3306 \
  -e APP_DB_NAME=portmaster \
  -e APP_DB_USER=portmaster \
  -e APP_DB_PASSWORD='...' \
  -e APP_JWT_SECRET='...' \
  <imagem>
```

A imagem roda como usuário não-root (`uid 10001`), expõe `8000` e já traz um
`HEALTHCHECK` que bate no `/info` — então `docker ps` mostra `healthy` sozinho,
sem você configurar nada.

**Se o banco está no host** e o contêiner precisa alcançá-lo,
`APP_DB_HOST=host.docker.internal` resolve no Docker Desktop (macOS e Windows).
No Linux, acrescente `--add-host=host.docker.internal:host-gateway`.

---

## Binário direto (Linux x86-64)

```sh
chmod +x portmaster-api-http

export APP_DB_HOST=127.0.0.1 APP_DB_PORT=3306
export APP_DB_NAME=portmaster APP_DB_USER=portmaster
export APP_DB_PASSWORD='...' APP_JWT_SECRET='...'

./portmaster-api-http
```

É estático: não procura biblioteca nenhuma do sistema em tempo de execução, e
roda igual em distribuições diferentes. `ldd` respondendo "not a dynamic
executable" é o esperado.

---

## Outras plataformas

### macOS (zsh/bash)

O binário é `linux/amd64` e **não** roda aqui. Use a imagem:

```sh
docker load -i portmaster-api-image.tar
docker run -d -p 8000:8000 \
  -e APP_DB_HOST=host.docker.internal \
  -e APP_DB_PORT=3306 \
  -e APP_DB_NAME=portmaster \
  -e APP_DB_USER=portmaster \
  -e APP_DB_PASSWORD='...' \
  -e APP_JWT_SECRET='...' \
  <imagem>
```

Em Apple Silicon a imagem é amd64 e roda sob emulação (Rosetta/QEMU). Funciona,
mas mais devagar — para desenvolver, considere compilar do fonte.

### Windows (PowerShell)

O binário também não roda aqui. Note que o PowerShell usa crase para continuar
linha, não a contrabarra:

```powershell
docker load -i .\portmaster-api-image.tar

docker run -d --name portmaster-api `
  -p 8000:8000 `
  -e APP_DB_HOST=host.docker.internal `
  -e APP_DB_PORT=3306 `
  -e APP_DB_NAME=portmaster `
  -e APP_DB_USER=portmaster `
  -e APP_DB_PASSWORD='...' `
  -e APP_JWT_SECRET='...' `
  <imagem>
```

Conferir que subiu:

```powershell
Invoke-RestMethod http://localhost:8000/info
```

Aspas simples no PowerShell não interpolam, que é o que você quer para uma senha
com `$` dentro. Com aspas duplas, `$` inicia uma variável e a senha chega
truncada ao contêiner.

### WSL

Vale a seção do binário direto: é Linux x86-64. Se o Docker Desktop estiver com a
integração WSL ligada, a seção do Docker vale igual.

---

## systemd

```ini
[Unit]
Description=Portmaster API
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=portmaster
ExecStart=/usr/local/bin/portmaster-api-http
Restart=on-failure
RestartSec=5

Environment=APP_HOST=0.0.0.0
Environment=APP_PORT=8000
Environment=APP_DB_HOST=127.0.0.1
Environment=APP_DB_PORT=3306
Environment=APP_DB_NAME=portmaster
Environment=APP_DB_USER=portmaster
EnvironmentFile=/etc/portmaster/secrets.env

NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=yes

[Install]
WantedBy=multi-user.target
```

Senha e segredo do JWT vão no `EnvironmentFile`, não em `Environment=`: o que
está na unit aparece em `systemctl show` para qualquer usuário do sistema.

`ProtectSystem=strict` funciona porque o processo não escreve em disco — ele fala
com o banco e com a rede, e nada mais.

---

## Migrations

O schema é aplicado à parte, com [golang-migrate](https://github.com/golang-migrate/migrate),
a partir do tarball de migrations:

```sh
migrate -path ./migrations \
        -database 'mysql://user:senha@tcp(host:3306)/portmaster' up
```

A API **não** aplica schema no boot. Ela espera encontrá-lo pronto, e é isso que
permite subir várias instâncias sem que duas tentem migrar ao mesmo tempo.
