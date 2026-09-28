//! Eliminación de producciones-ε, mostrando cada fase del algoritmo.

use crate::lector::{Cuerpo, Gramatica};
use crate::pantalla::{self, GRIS, MORADO, NEGRITA, RESET, ROJO, VERDE};

/// Fase 1: punto fijo. Un símbolo es anulable si tiene A → ε,
/// o A → X1...Xk con todos los Xi anulables.
fn buscar_anulables(g: &Gramatica) -> Vec<char> {
    let mut anulables: Vec<char> = Vec::new();
    let mut ronda = 1;

    loop {
        let mut encontrados = Vec::new();
        for (cabeza, cuerpos) in &g.reglas {
            if anulables.contains(cabeza) {
                continue;
            }
            let razon = cuerpos.iter().find(|c| c.iter().all(|s| anulables.contains(s)));
            if let Some(c) = razon {
                encontrados.push((*cabeza, c.clone()));
            }
        }

        if encontrados.is_empty() {
            println!("   Ronda {ronda}: sin cambios → el conjunto ya no crece, se termina.");
            break;
        }

        let motivos: Vec<String> = encontrados
            .iter()
            .map(|(a, c)| format!("{a} (por {})", pantalla::regla(*a, std::slice::from_ref(c))))
            .collect();
        anulables.extend(encontrados.iter().map(|(a, _)| *a));
        println!(
            "   Ronda {ronda}: se agregan {}   ⇒  N = {}",
            motivos.join(", "),
            pantalla::conjunto(&anulables)
        );
        ronda += 1;
    }
    anulables
}

pub fn quitar_epsilon(g: &Gramatica) -> Gramatica {
    pantalla::encabezado("Eliminación de producciones-ε");

    pantalla::fase(1, "Símbolos anulables (A ⇒* ε)");
    println!("   {GRIS}N empieza vacío: N = {{}}{RESET}");
    let anulables = buscar_anulables(g);
    println!("   {VERDE}{NEGRITA}Anulables: N = {}{RESET}", pantalla::conjunto(&anulables));

    pantalla::fase(2, "Producciones anulables (su cuerpo puede volverse ε)");
    let mut alguna = false;
    for (cabeza, cuerpos) in &g.reglas {
        for c in cuerpos.iter().filter(|c| c.iter().all(|s| anulables.contains(s))) {
            alguna = true;
            let nota = if c.is_empty() { "es ε directamente" } else { "todo su cuerpo está en N" };
            println!("   • {}   {GRIS}← {nota}{RESET}", pantalla::regla(*cabeza, std::slice::from_ref(c)));
        }
    }
    if !alguna {
        println!("   • ninguna");
    }

    pantalla::fase(3, "Nuevas producciones: 2^m combinaciones por regla");
    println!("   {GRIS}Cada bit indica si el anulable se quita (1) o se deja (0).{RESET}");

    let mut limpia = Gramatica::nueva(g.inicial);
    for (cabeza, cuerpos) in &g.reglas {
        for original in cuerpos {
            let titulo = pantalla::regla(*cabeza, std::slice::from_ref(original));
            if original.is_empty() {
                println!("\n   {titulo}  {ROJO}✗ es una producción-ε, se quita{RESET}");
                continue;
            }

            let posiciones: Vec<usize> =
                (0..original.len()).filter(|&i| anulables.contains(&original[i])).collect();
            let m = posiciones.len();
            let total = 1usize << m;
            let cuales: Vec<String> = posiciones.iter().map(|&i| original[i].to_string()).collect();
            println!(
                "\n   {titulo}   {GRIS}m = {m}{}  →  2^{m} = {total} combinación(es){RESET}",
                if m > 0 { format!(" ({})", cuales.join(", ")) } else { String::new() }
            );

            for mascara in 0..total {
                let quitar: Vec<usize> =
                    (0..m).filter(|b| mascara & (1 << b) != 0).map(|b| posiciones[b]).collect();
                let nuevo: Cuerpo = original
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| !quitar.contains(i))
                    .map(|(_, s)| *s)
                    .collect();

                let bits: String = if m == 0 {
                    "-".into()
                } else {
                    (0..m).map(|b| if mascara & (1 << b) != 0 { '1' } else { '0' }).collect()
                };

                let resultado = if nuevo.is_empty() {
                    format!("{ROJO}ε → se descarta{RESET}")
                } else if nuevo.len() == 1 && nuevo[0] == *cabeza {
                    format!("{ROJO}{cabeza} → {cabeza} no aporta nada, se descarta{RESET}")
                } else {
                    let texto = pantalla::regla(*cabeza, std::slice::from_ref(&nuevo));
                    if limpia.agregar(*cabeza, nuevo) {
                        format!("{VERDE}{texto}{RESET}")
                    } else {
                        format!("{GRIS}{texto} (ya estaba){RESET}")
                    }
                };
                println!("      [{bits:>width$}]  {resultado}", width = m.max(1));
            }
        }
    }

    if anulables.contains(&g.inicial) {
        println!(
            "\n   {MORADO}Ojo: {} ∈ N, o sea que ε ∈ L(G). La gramática nueva genera L(G) − {{ε}}.{RESET}",
            g.inicial
        );
    }

    pantalla::gramatica("Resultado — gramática sin producciones-ε:", &limpia);
    limpia
}
