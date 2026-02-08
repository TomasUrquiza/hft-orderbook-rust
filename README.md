# HFT Orderbook Engine in Rust 🦀

Un motor de emparejamiento de órdenes (Matching Engine) de alta frecuencia diseñado para la eficiencia y la seguridad de memoria.

## 🚀 Features
* **High Performance:** Estructuras de datos `BTreeMap` para ordenamiento en $O(\log n)$.
* **Async Core:** Arquitectura no bloqueante usando `Tokio` channels.
* **Type Safety:** Manejo monetario preciso con `rust_decimal` (sin errores de punto flotante).

## 🛠 Tech Stack
* **Lenguaje:** Rust
* **Concurrency:** Tokio (Actor Model pattern)
* **Math:** Rust Decimal

## ⚡ Quick Start
```bash
cargo run
