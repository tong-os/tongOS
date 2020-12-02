# tongOS

Projeto final para a disciplina INE5424 - Sistemas Operacionais II. **tongOS** é um Sistema Operacional baseado em RISC-V e feito em Rust. 

Principal referência de implementação: Stephen Marz¹.

## Versão
0.5


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
1. Corretude na execução da política de escalonamento particionado.
2. Corretude na migração de threads.
3. Corretude na política de migração implementada.

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

A inicialização das outras harts já era feito anteriormente no processo de `boot`. Essa parte está em `entry.S`. Adicionalmente, adicionamos, na função `kinit()` em `main.rs`, a "finalização" do setup das outras harts. Todas as harts, com exceção da 0, esperam até que a **hart 0** termine a inicialização do sistema e acorde-as, através da variável `MAY_BOOT`, permitindo que escalonem algum processo. 

Adaptamos o sistema para mostar, ao printar, a hart corrente, a hart anterior e o pid do processo que está realizando essa saída.

Assim como na entrega 4, cada hart possui sua fila de processos. A migração de processos entre harts é realizada sempre que um processo transite de um estado qualquer (`running, blocked, sleeping`) para `ready`. Esse procedimento é realizado na função `migrate_process`, localizada em `process.rs`. Nela, é invocada uma função que decide para qual hart o processo será migrado: `migration_criteria`, localizada em `scheduler.rs`.

Foram implementadas três políticas de migração bem simples: adição via mod, Round Robin e "disponibilidade". É importante ressaltar que a API para adicionar um novo critério é bem simples, bastando apenas criar uma função e adicioná-la na lista de critérios disponíveis, selecionados pela variável `CRITERIA`, localizada em `scheduler.rs`. Para a primeira, apenas adicionamos 1 no valor da hart corrente e realizamos a operação de % 4, para que fique no intervalo adequado. Para a segunda, existe uma variável chamada `NEXT_HART`, compartilhada por todas as harts, que é adicionada de um sempre que chamada, fazendo % 4 no final. Para a terceira, primeiro olha-se se alguma hart está executando `IDLE`, senão busca a hart com a menor fila `ready`. 




## Como debugar
```
Primeiro terminal: make run_debug -> executa o qemu em forma de debugação.
Segundo terminal: make debug -> inicia o debug, carregando os símbolos e dando target em localhost:1234.
```

## Referências
¹ Stephen Marz. Tutorial: https://osblog.stephenmarz.com/ e repositório: https://github.com/sgmarz/osblog. Arquivos ou funções específicas poderão conter referência direta a ele.
