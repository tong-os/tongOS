# tongOS

Projeto final para a disciplina INE5424 - Sistemas Operacionais II. **tongOS** é um Sistema Operacional baseado em RISC-V e feito em Rust. 

Principal referência de implementação: Stephen Marz¹.

## Versão
0.1

## Como rodar

## Instalação
```
Padrão: https://doc.rust-lang.org/book/ch01-01-installation.html
Arch/Manjaro: https://wiki.archlinux.org/index.php/rust#Installation
```
## Pré requisitos
```
rustup default nightly
rustup target add riscv64gc-unknown-none-elf
cargo install cargo-binutils
```

### Sistema
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
[1]: as seções inicializadas são impressas na tela, além de testar se o espaço de `bss` está todo zerado.
[2]: a MMU é inicializada e testada a partir da impressão das alocações e tabelas de páginas. São realizadas algumas alocações para teste.
[3]: o funcionamento da UART é conferido na totalidade dos outros testes, igualmente.
[4]: 
[5]: 

## Referências

¹ Stephen Marz. Tutorial: https://osblog.stephenmarz.com/ e repositório: https://github.com/sgmarz/osblog. Arquivos ou funções específicas poderão conter referência direta a ele.

