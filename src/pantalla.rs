//! Todo lo relacionado con imprimir en la terminal.

use crate::lector::{Cuerpo, Gramatica, VACIA};

pub const RESET: &str = "\x1b[0m";
pub const GRIS: &str = "\x1b[90m";
pub const ROJO: &str = "\x1b[91m";
pub const VERDE: &str = "\x1b[92m";
pub const AMARILLO: &str = "\x1b[93m";
pub const AZUL: &str = "\x1b[94m";
pub const MORADO: &str = "\x1b[95m";
pub const NEGRITA: &str = "\x1b[1m";

pub fn cuerpo(c: &Cuerpo) -> String {
    if c.is_empty() {
        VACIA.to_string()
    } else {
        c.iter().collect()
    }
}

pub fn regla(cabeza: char, cuerpos: &[Cuerpo]) -> String {
    let lista: Vec<String> = cuerpos.iter().map(cuerpo).collect();
    format!("{cabeza} → {}", lista.join(" | "))
}

pub fn conjunto(simbolos: &[char]) -> String {
    let lista: Vec<String> = simbolos.iter().map(char::to_string).collect();
    format!("{{{}}}", lista.join(", "))
}

pub fn gramatica(titulo: &str, g: &Gramatica) {
    println!("\n{NEGRITA}{titulo}{RESET}");
    println!("   ┌──────────────────────────────────");
    for (cabeza, cuerpos) in &g.reglas {
        if !cuerpos.is_empty() {
            println!("   │ {VERDE}{}{RESET}", regla(*cabeza, cuerpos));
        }
    }
    println!("   └──────────────────────────────────");
}

pub fn encabezado(texto: &str) {
    println!("\n{AZUL}{NEGRITA}╔{}╗{RESET}", "═".repeat(58));
    println!("{AZUL}{NEGRITA}║  {texto:<56}║{RESET}");
    println!("{AZUL}{NEGRITA}╚{}╝{RESET}", "═".repeat(58));
}

pub fn fase(numero: u8, texto: &str) {
    println!("\n{AMARILLO}{NEGRITA}[Fase {numero}] {texto}{RESET}");
}
