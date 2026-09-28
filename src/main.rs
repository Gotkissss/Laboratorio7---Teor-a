//! Laboratorio 7 — Teoría de la Computación
//! Lee gramáticas desde archivos, valida cada línea con un AFN construido
//! desde una expresión regular y elimina las producciones-ε.
//!
//! Uso:  cargo run -q -- entradas/g1.txt [otro.txt ...]

mod automata;
mod epsilon;
mod lector;
mod pantalla;

use lector::{Fallo, PATRON};
use pantalla::{GRIS, MORADO, NEGRITA, RESET, ROJO, VERDE};
use std::process::exit;

fn main() {
    let archivos: Vec<String> = std::env::args().skip(1).collect();
    if archivos.is_empty() {
        println!("{NEGRITA}Uso:{RESET} cargo run -q -- <archivo.txt> [más archivos]");
        println!("  cargo run -q -- entradas/g1.txt");
        println!("  cargo run -q -- entradas/g1.txt entradas/g2.txt entradas/g3.txt");
        println!("  cargo run -q -- entradas/con_errores/falta_flecha.txt");
        exit(2);
    }

    let afn = automata::compilar(PATRON).unwrap_or_else(|e| {
        println!("{ROJO}El patrón de validación está mal: {e}{RESET}");
        exit(3);
    });

    for ruta in &archivos {
        pantalla::encabezado(&format!("Archivo: {ruta}"));
        println!("\n{NEGRITA}Validación de líneas{RESET}");
        println!("   Regex: {GRIS}{PATRON}{RESET}");
        println!("   {GRIS}(convertida a un AFN de Thompson con {} estados){RESET}", afn.cantidad_estados());

        let lectura = match lector::leer(ruta, &afn) {
            Ok(l) => l,
            Err(fallo) => {
                reportar(fallo);
                println!("\n{ROJO}{NEGRITA}■ Ejecución detenida: no se puede continuar.{RESET}\n");
                exit(1);
            }
        };

        for (n, texto) in &lectura.buenas {
            println!("   {VERDE}✓{RESET} línea {n}: {texto}");
        }
        println!("   {VERDE}Todas las líneas son producciones válidas.{RESET}");
        if !lectura.sin_reglas.is_empty() {
            println!(
                "   {MORADO}Nota: {} se usa en algún cuerpo pero no tiene producciones.{RESET}",
                pantalla::conjunto(&lectura.sin_reglas)
            );
        }

        pantalla::gramatica(
            &format!("Gramática cargada (inicial: {})", lectura.gramatica.inicial),
            &lectura.gramatica,
        );
        epsilon::quitar_epsilon(&lectura.gramatica);
    }
    println!();
}

fn reportar(fallo: Fallo) {
    match fallo {
        Fallo::NoSeAbre(ruta) => println!("   {ROJO}✗ No se encontró o no se pudo leer '{ruta}'.{RESET}"),
        Fallo::SinProducciones => println!("   {ROJO}✗ El archivo no tiene ninguna producción.{RESET}"),
        Fallo::Linea { buenas, mala } => {
            for (n, texto) in &buenas {
                println!("   {VERDE}✓{RESET} línea {n}: {texto}");
            }
            let prefijo = format!("   ✗ línea {}: ", mala.numero);
            println!("{ROJO}{prefijo}{}{RESET}", mala.texto);
            println!(
                "{ROJO}{}^ el autómata se detiene aquí (columna {}){RESET}",
                " ".repeat(prefijo.chars().count() + mala.columna),
                mala.columna + 1
            );
            println!("   {ROJO}Motivo: {}{RESET}", mala.motivo);
        }
    }
}
