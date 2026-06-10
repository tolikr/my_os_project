# Следование различным гайдам, не понятно что получится.

## Подготовка окружения

1. Поставить wsl если на винде
(Часть дальнейшего взята из https://jcomes.org/aarch64-os-hello_world)

2. Установить rust

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

3. QEMU RISC-v: 

```
sudo apt install qemu-system-riscv
```

4. Компилятор:

```
sudo apt install binutils-riscv64-unknown-elf
```

5. rust таргет:

```
rustup target add riscv64gc-unknown-none-elf
```

## Лицензия

Проект распространяется под лицензией MIT. Подробнее см. в файле [LICENSE](LICENSE).