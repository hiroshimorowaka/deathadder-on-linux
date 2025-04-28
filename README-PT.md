## Razer DeathAdder V2 Linux Fix

### Um driver muito simples apenas para remapear os botões DPI do rato.

- [Razer DeathAdder V2 Linux Fix](#razer-deathadder-v2-linux-fix)
    - [Um driver muito simples apenas para remapear os botões DPI do rato.](#um-driver-muito-simples-apenas-para-remapear-os-bot%C3%B5es-dpi-do-rato)
- [Dependências](#depend%C3%AAncias)
- [Configurando o código](#configurando-o-c%C3%B3digo)
- [Executando o código](#executando-o-c%C3%B3digo)
- [Executando como um serviço](#executando-como-um-servi%C3%A7o)
    - [Criando um serviço](#criando-um-servi%C3%A7o)
    - [Permissões](#permiss%C3%B5es)
    - [Adicionando o serviço](#adicionando-o-servi%C3%A7o)
- [Contribuindo](#contribuindo)


Este driver foi construído para resolver um problema muito específico que eu tinha: **os botões DPI do Deathadder**.

Eu tenho, obviamente, um Razer DeathAdder V2 como meu mouse principal, e eu uso os botões DPI para mutar e desmutar no discord. No Windows, configurei os botões de DPI no Razer Synapse para serem as funções `F23` e `F24` do teclado, mas no meu Linux (Zorin OS + Xorg) ele não identificou esses botões, e em vez de configurar o Xorg corretamente, fiz um “driver” para resolver esse problema.

Este “driver” se conecta na interface 2 do mouse, desconecta o driver do Kernel e intercepta as comunicações. Pegando os bytes, descobri que ele envia, no terceiro byte, qual o botão que está sendo pressionado, no caso do `F23` e `F24` que configurei, são, respectivamente, byte `114` e `115`. 

Intercepto esta comunicação e altero o botão que está a ser pressionado para `F14` e `F15`, o que o Xorg interpreta corretamente.

Ao transformar este programa num serviço, consegui resolver o meu problema e agora, apertando os mesmos botões do mouse, sem alterar as minhas configurações do Windows, posso mutar e desmutar no Discord sem qualquer problema.

## Dependências

Primeiro, precisamos instalar as dependências do Linux. 

```bash
sudo apt install libxtst-dev
sudo apt install libxdo-dev
```

## Configurando o código

O código precisa ser adaptado ao seu contexto. No meu caso, eu uso as teclas `F23` e `F24` já configuradas na memória do meu DeathAdder em vez dos botões de DPI, por isso você vai precisar de fazer a mesma coisa se não quiser alterar o código. 

`keyboard_buttons.rs`

```rust
pub enum KeyboardButtons {
    F23,
    F24,
}

impl KeyboardButtons {
    pub fn from_code(code: u8) -> Opção<KeyboardButtons> {
        corresponde ao código {
            114 => Some(KeyboardButtons::F23),
            115 => Some(KeyboardButtons::F24),
            _ => Nenhum,
        }
    }
}
```

Também terá de alterar o botão que será remapeado, no meu caso coloquei `PrintScreen` e `ScrollLock`, mas eles mapeiam para `F14` e `F15`. 


`main.rs`
```rust
    // Criar uma thread para manipular os pacotes
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        while let Ok(button) = rx.recv() {
            corresponder ao botão {
                KeyboardButtons::F23 => {
                    autopilot::key::tap(&Code(KeyCode::PrintScreen), &[], 1, 0);
                }
                KeyboardButtons::F24 => {
                    piloto automático::tecla::toque(&Code(KeyCode::ScrollLock), &[], 1, 0);
                }
            }
        }
    });
```



## Executando o código

No meu caso, prefiro executar meu código como `root`, já que é preciso privilégios de root para acessar os dispositivos USB, então executo o seguinte comando, dentro da pasta `/target/debug` do projeto.

```bash
cargo build && sudo ./razer_deathadder_v2_buttons_control
```

## Executando como um serviço

Para rodar o programa como um serviço dentro do linux, será necessário criar um novo serviço no `systemd`.

### Criando um serviço

Usando o seguinte comando, podemos criar um novo arquivo dentro dos serviços do `systemd`

```bash
sudo nano /etc/systemd/system/nome_do_seu_servico.service
```

No seu arquivo de serviço, o seguinte conteúdo é necessário:

```toml
[Unit]
Description=Fix DPI buttons not working in linux
After=network.target

[Service]
ExecStart=/usr/bin/razer_deathadder_v2_buttons_control
WorkingDirectory=/usr/bin
Restart=always
User=root
Group=root
Environment=PATH=/usr/bin:/bin
Environment=LD_LIBRARY_PATH=/lib:/usr/lib
Environment=DISPLAY=:0
Environment=XAUTHORITY=/home/your_user_name/.Xauthority
StandardOutput=journal
StandardError=journal

# Reattaching driver after service stop
ExecStopPost=/bin/bash -c '/usr/bin/razer_deathadder_v2_buttons_control -r'

[Install]
WantedBy=multi-user.target
```

> **OBS**: a linha `ExecStopPost=/bin/bash -c '/usr/bin/razer_deathadder_v2_buttons_control -r'` é usada para que, quando o serviço parar, ele reconecte o driver ao kernel, para evitar que os botões parem de funcionar.

### Permissões

No terminal, é preciso dar permissão ao root para que ele possa acessar o `servidor X`

```bash
xhost +SI:utilizador local:root
```

### Adicionando o serviço

```bash
# Reiniciando o daemon
sudo systemctl daemon-reload

# Habilitando o serviço na inicialização
sudo systemctl enable nome_do_seu_servico.service

# Iniciando o serviço manualmente
sudo systemctl start nome_do_seu_servico.service
```

Você também pode verificar os logs do seu serviço para debugging

```bash
journalctl -u nome_do_seu_servico.service -f
```

Seu serviço agora está configurado para iniciar com o sistema e já está em execução.

## Contribuindo

Se você estiver interessado em contribuir com o projeto, será muito bem-vindo. Basta abrir uma issue e seu PR e vamos conversar