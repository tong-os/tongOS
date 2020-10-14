# tongOS seminario MP Interrupts

## Pré requisitos

Instalar o rustup https://rustup.rs/

```
rustup install nightly
rustup target add riscv64gc-unknown-none-elf
```

Instalar o qemu-system-riscv64 

## Como rodar

Primeiro crie o hdd com o comando make
```
make
```

Para executar, execute na pasta root do projeto
```
cargo run
```
