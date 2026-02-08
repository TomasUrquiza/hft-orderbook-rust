use std::collections::BTreeMap;
use std::cmp::Ordering;
use rust_decimal::prelude::*; // Necesario para manejar precios financieros
use tokio::sync::mpsc;        // Canales para comunicación asíncrona
use tokio::time::{sleep, Duration};

// --- ESTRUCTURAS DE DATOS ---

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    pub id: u64,
    pub price: Decimal,
    pub amount: Decimal,
    pub side: Side,
    pub timestamp: u64,
}

// --- LÓGICA DE ORDENAMIENTO (EL MOTOR MATEMÁTICO) ---

impl Ord for Order {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.side {
            // Compras: Prioridad al precio MÁS ALTO. Desempate por tiempo (FIFO).
            Side::Buy => other.price.cmp(&self.price)
                .then_with(|| self.id.cmp(&other.id)), // Usamos ID como proxy de tiempo simple
            
            // Ventas: Prioridad al precio MÁS BAJO. Desempate por tiempo (FIFO).
            Side::Sell => self.price.cmp(&other.price)
                .then_with(|| self.id.cmp(&other.id)),
        }
    }
}

impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// --- EL LIBRO DE ÓRDENES ---

pub struct OrderBook {
    bids: BTreeMap<Order, Decimal>, // Compras
    asks: BTreeMap<Order, Decimal>, // Ventas
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn add_order(&mut self, mut order: Order) {
        println!("--> 📥 Recibida Orden #{}: {:?} {} @ {}", order.id, order.side, order.amount, order.price);

        // Lógica de Matching (Cruce)
        loop {
            if order.amount <= Decimal::zero() { break; } // Orden completada

            let match_found = match order.side {
                Side::Buy => {
                    // Si compro, busco la venta más barata (asks)
                    if let Some((best_ask, ask_amount)) = self.asks.iter_mut().next() {
                        if best_ask.price <= order.price {
                            // ¡MATCH!
                            let trade_amount = order.amount.min(*ask_amount);
                            println!("   ⚡ MATCH EJECUTADO: Compra #{} vs Venta #{} :: Cantidad {}", order.id, best_ask.id, trade_amount);
                            
                            // Actualizar cantidades (lógica simplificada)
                            order.amount -= trade_amount;
                            *ask_amount -= trade_amount;
                            
                            // Si la orden del libro se agotó, habría que eliminarla (aquí omitido por brevedad)
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                },
                Side::Sell => {
                    // Si vendo, busco la compra más cara (bids)
                    if let Some((best_bid, bid_amount)) = self.bids.iter_mut().next() {
                        if best_bid.price >= order.price {
                            // ¡MATCH!
                            let trade_amount = order.amount.min(*bid_amount);
                            println!("   ⚡ MATCH EJECUTADO: Venta #{} vs Compra #{} :: Cantidad {}", order.id, best_bid.id, trade_amount);
                            
                            order.amount -= trade_amount;
                            *bid_amount -= trade_amount;
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
            };

            if !match_found {
                break; // No hay más matches posibles, salir del loop
            }
            // Si hubo match, el loop continúa para intentar llenar el resto de la orden
            break; // BREAK TEMPORAL: Para evitar loops infinitos si no borramos las órdenes en 0.
        }

        // Si sobra cantidad, guardar en el libro
        if order.amount > Decimal::zero() {
            println!("   📌 Guardando resto en el libro: {} @ {}", order.amount, order.price);
            match order.side {
                Side::Buy => { self.bids.insert(order.clone(), order.amount); },
                Side::Sell => { self.asks.insert(order.clone(), order.amount); },
            };
        }
    }
}

// --- ARQUITECTURA ASÍNCRONA (SYSTEMS ENGINEERING) ---

#[tokio::main]
async fn main() {
    println!("🚀 Iniciando HFT Engine v1.0...");

    // 1. Canal de comunicación: Gateway -> Engine
    let (tx, mut rx) = mpsc::channel(100);

    // 2. Spawn del Motor (Consumer) en su propio hilo verde
    let engine_handle = tokio::spawn(async move {
        let mut book = OrderBook::new();
        while let Some(order) = rx.recv().await {
            book.add_order(order);
        }
    });

    // 3. Simulación de Tráfico (Producer)
    let orders = vec![
        // Vendedor pone 1 BTC a 50,000
        Order { id: 1, price: Decimal::from(50000), amount: Decimal::from(1), side: Side::Sell, timestamp: 100 },
        // Comprador pone orden baja a 49,000 (No match)
        Order { id: 2, price: Decimal::from(49000), amount: Decimal::from(1), side: Side::Buy, timestamp: 101 },
        // Comprador agresivo a 51,000 (Debería matchear con la venta #1)
        Order { id: 3, price: Decimal::from(51000), amount: Decimal::from(2), side: Side::Buy, timestamp: 102 },
    ];

    for order in orders {
        tx.send(order).await.unwrap();
        sleep(Duration::from_millis(500)).await; // Pequeña pausa para ver el efecto
    }

    println!("✅ Todas las órdenes enviadas. Cerrando canal...");
    drop(tx); // Cierra el canal
    engine_handle.await.unwrap(); // Espera a que el motor termine de procesar
}