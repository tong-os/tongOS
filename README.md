# tongOS

Projeto final para a disciplina INE5424 - Sistemas Operacionais II. **tongOS** é um Sistema Operacional baseado em RISC-V e feito em Rust. 

Principal referência de implementação: Stephen Marz¹.

## Versão
0.2


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
1. Corretude na inicialização e configuração do(s) Timer(s), incluindo handler com eoi e configuração de alarmes.
2. Corretude no tratamento de interrupções.
3. Demonstrando o funcionamento do escalonador com preempção por Timer, com possibilidade de configuração do tempo necessário para preempção

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
Para habilitar a preempção, criamos a variável `ENABLE_PREEMPTION`, também em `lib.rs`. Quando habilitada, o sistema tratará (e escalonará) 
as interrupções de relógio. O tempo de interrupção de relógio é definido por (`CONTEXT_SWITCH_TIME` * `process.quantum`). É possível alterar o
`CONTEXT_SWITCH_TIME` em `cpu.rs` e o `quantum` do processo em `process.rs`. Atualmente, todos os processos são inicializados com o mesmo `quantum` padrão.
O escalonador funciona como `Round-Robin` quando a preempção está habilitada; caso contrário, funciona como `First Come, First Served`.

Para testar interrupções externas, criamos uma `syscall` para receber inputs do teclado, através do `read_line`. Essa `syscall` habilita a PLIC para que 
seja possível receber interrupções externas, posteriormente tratadas em `trap.rs`. O app [3] demonstra a execução. 

## Como debugar
```
Primeiro terminal: make run_debug -> executa o qemu em forma de debugação.
Segundo terminal: make debug -> inicia o debug, carregando os símbolos e dando target em localhost:1234.
```

## Referências
¹ Stephen Marz. Tutorial: https://osblog.stephenmarz.com/ e repositório: https://github.com/sgmarz/osblog. Arquivos ou funções específicas poderão conter referência direta a ele.
