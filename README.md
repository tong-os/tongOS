# tongOS

Projeto final para a disciplina INE5424 - Sistemas Operacionais II. **tongOS** é um Sistema Operacional baseado em RISC-V e feito em Rust. 

Principal referência de implementação: Stephen Marz¹.

## Versão
0.1


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
Para a primeira entrega do projeto, é necessário cobrir os seguintes tópicos:
1. Corretude na inicialização do Sistema Operacional. 
2. Corretude na configuração da cache e inicialização da MMU.
3. Funcionamento da Saída (impressão) através do Console Qemu.
4. Corretude na troca de contexto no processador.
5. Demonstração do funcionamento do escalonador.

## Visualização
Os testes para a primeira entrega estão apresentados no arquivo `main.rs`, chamados pela função `kinit()`.
1. as seções inicializadas são impressas na tela, além de testar se o espaço de `bss` está todo zerado.
2. a MMU é inicializada e testada a partir da impressão das alocações e tabelas de páginas. São realizadas algumas alocações para teste.
3. o funcionamento da UART é conferido na totalidade dos outros testes, igualmente.
4. está sendo realizado a troca de contexto entre os processos de exemplo 1, 2 e __philosophers_diner__.
5. os processos são escalonados em forma de fila, FIFO. Temos um __join__ bloqueante para trabalhar um processo como se fosse uma __thread__.

## Pontos importantes para [4] e [5]
Deixamos alguns comentários para facilitar a debugação, mas são faiclmente removidos se assim desejado.
Em `kinit()`, no arquuivo `main.rs`, são inicializados três processos para demonstrar o funcionamento do escalonador.
Os dois primeiros são apenas para imprimir coisas simples na tela e o terceiro inicia o __philosophers_diner__, seguindo a implementação presente no EPOS.

Para o funcionamento de __philosophers_diner__, desenvolvemos uma forma de simular a presença de __threads__, mas ainda com processos. Isso é feito a partir
do __join__ bloqueante e de um tratamento especial em `trap.rs`, utilizando-se da execeção __ECALL__.

## Como debugar
```
Primeiro terminal: make run_debug -> executa o qemu em forma de debugação.
Segundo terminal: make debug -> inicia o debug, carregando os símbolos e dando target em localhost:1234.
```

## Referências

¹ Stephen Marz. Tutorial: https://osblog.stephenmarz.com/ e repositório: https://github.com/sgmarz/osblog. Arquivos ou funções específicas poderão conter referência direta a ele.
