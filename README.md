# tongOS

Projeto final para a disciplina INE5424 - Sistemas Operacionais II. **tongOS** é um Sistema Operacional baseado em RISC-V e feito em Rust. 

Principal referência de implementação: Stephen Marz¹.

## Versão
0.4


## Instalação
```
Padrão: https://doc.rust-lang.org/book/ch01-01-installation.html ou https://rustup.rs/
Arch/Manjaro: https://wiki.archlinux.org/index.php/rust#Installation
```
## Pré requisitos
```
rustup install nightly
rustup target add riscv64gc-unknown-none-elf
Talvez seja necessário -> (cargo install cargo-binutils)
```

## Como rodar
Para executar, mesmo após qualquer mudança, basta o seguinte comando: 
```
cargo run
```
Para pontos de entrega e como visualizar, veja __entrega__ e __visualização__.

## Entrega
Para a segunda entrega do projeto, é necessário cobrir os seguintes tópicos:
1. Corretude do Sistema Operacional na execução com processador multicore
2. Corretude na execução da política de escalonamento particionado: "Threads criadas em uma partição sejam sempre alocadas ao mesmo core".

## Visualização
Os testes para a segunda entrega estão apresentados no arquivo `assigment.rs`, chamados pela função `kinit()` em `main.rs`.
Para esta entrega, apresentamos duas novas features de visualização: `DEBUG_OUTPUT` e `PROCESS_TO_RUN`, ambas definidas em `lib.rs`.
Com `DEBUG_OUTPUT` é possível ativar ou desativar os prints de debug no meio do código (as vezes pode ficar meio difícil de entender o que está acontecendo).
Com `PROCESS_TO_RUN` você pode escolher qual processo/app executar. As opções são:
1. Processos simples de exemplo: impressões e alguns loops com somador.
2. Jantar dos Filósofos.
3. App simples com input de teclado + sleep.
4. Executar todos em sequência.


## Pontos importantes para a entrega
A execução com 4 harts está **hardcoded**, por algumas razões. É possível verificar que o `qemu` chama `-smp 4` em `.cargo/config`. 

A inicialização das outras harts já era feito anteriormente no processo de `boot`. Essa parte está em `entry.S`. Adicionalmente, adicionamos, na função `kinit()` em `main.rs`, a "finalização" do setup das outras harts. Todas as harts, com exceção da 0, esperam até que a **hart 0** termine a inicialização do sistema e acorde-as, através da variável `MAY_BOOT`, permitindo que escalonem algum processo. Para não atrapalhar o funcionamento de outras harts, não são executadas interrupções entre harts no momento, mesmo que tenhamos o suporte para tal (`wake_all_idle_harts`, comentada por hora).

Adaptamos o sistema para mostar, ao printar, a hart corrente e o pid do processo que está realizando essa saída.

Para esta entrega, decidimos executar o mesmo processo em todas as harts, mas de forma independente. Desse modo, cada hart possui suas próprias filas: `ready, blocked e sleeping`. Além disso, contamos com 4 espaços reservados para processos `running` e `idle`, de suas respectivas harts.

Cada hart possui `locks` respectivos para suas próprias filas e listas. Desse modo, ao interagir, apenas  as que estiverem em seu comando serão bloqueadas e manipuladas. É possível visualizar este funcionamento atráves do Jantar dos Filósofos, verificando que os pids são propriedade apenas de uma hart específica.


## Como debugar
```
Primeiro terminal: make run_debug -> executa o qemu em forma de debugação.
Segundo terminal: make debug -> inicia o debug, carregando os símbolos e dando target em localhost:1234.
```

## Referências
¹ Stephen Marz. Tutorial: https://osblog.stephenmarz.com/ e repositório: https://github.com/sgmarz/osblog. Arquivos ou funções específicas poderão conter referência direta a ele.
