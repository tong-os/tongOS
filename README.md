# tongOS

Projeto final para a disciplina INE5424 - Sistemas Operacionais II. **tongOS** é um Sistema Operacional baseado em RISC-V e feito em Rust. 

Principal referência de implementação: Stephen Marz¹.

## Versão
0.1

## Pré requisitos
```
rustup default nightly
rustup target add riscv64gc-unknown-none-elf
cargo install cargo-binutils
```

## Como rodar

### Criação do HDD

### Sistema
Existem algumas formas de  rodar, de acordo com o que é desejado ser exibido. 
É possível selecionar qual teste a ser rodado, de acordo com os itens da entrega em questão. Veja a seção __entrega__ para mais detalhes.
```
make run test=1
make run all
```

## Entrega
Para a primeira entrega do projeto, é necessário cobrir os seguintes tópicos:
1. Corretude na inicialização do Sistema Operacional. 
2. Corretude na configuração da cache e inicialização da MMU.
3. Funcionamento da Saída (impressão) através do Console Qemu.
4. Corretude na troca de contexto no processador.
5. Demonstração do funcionamento do escalonador.



## Referências

¹ Stephen Marz. Tutorial: https://osblog.stephenmarz.com/ e repositório: https://github.com/sgmarz/osblog. Arquivos ou funções específicas poderão conter referência direta a ele.

