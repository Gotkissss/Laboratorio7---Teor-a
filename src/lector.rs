//! Lectura de archivos de gramáticas y validación de cada línea con el AFN.

use crate::automata::{Afn, Veredicto};
use std::fs;

pub const VACIA: char = 'ε';

/// Patrón de una línea de producción:
///   cabeza mayúscula, flecha (-> o →), uno o más cuerpos separados por |.
///   Un cuerpo son letras/dígitos o una ε sola. Se permiten espacios.
pub const PATRON: &str =
    " *[A-Z] *(->|→) *([a-zA-Z0-9]+|ε)( *\\| *([a-zA-Z0-9]+|ε))* *";

pub type Cuerpo = Vec<char>; // vacío = ε

pub struct Gramatica {
    pub inicial: char,
    pub reglas: Vec<(char, Vec<Cuerpo>)>, // en el orden del archivo
}

impl Gramatica {
    pub fn nueva(inicial: char) -> Self {
        Gramatica { inicial, reglas: Vec::new() }
    }

    /// Agrega cabeza -> cuerpo si todavía no existe. Devuelve si fue nueva.
    pub fn agregar(&mut self, cabeza: char, cuerpo: Cuerpo) -> bool {
        let i = match self.reglas.iter().position(|(c, _)| *c == cabeza) {
            Some(i) => i,
            None => {
                self.reglas.push((cabeza, Vec::new()));
                self.reglas.len() - 1
            }
        };
        let cuerpos = &mut self.reglas[i].1;
        if cuerpos.contains(&cuerpo) {
            return false;
        }
        cuerpos.push(cuerpo);
        true
    }

    pub fn tiene_reglas(&self, simbolo: char) -> bool {
        self.reglas.iter().any(|(c, _)| *c == simbolo)
    }
}

pub struct LineaMala {
    pub numero: usize,
    pub texto: String,
    pub columna: usize,
    pub motivo: String,
}

pub enum Fallo {
    NoSeAbre(String),
    SinProducciones,
    Linea { buenas: Vec<(usize, String)>, mala: LineaMala },
}

pub struct Lectura {
    pub gramatica: Gramatica,
    pub buenas: Vec<(usize, String)>,
    pub sin_reglas: Vec<char>,
}

pub fn es_variable(c: char) -> bool {
    c.is_ascii_uppercase()
}

fn buscar_flecha(linea: &str) -> Option<(usize, usize)> {
    linea
        .find("->")
        .map(|i| (i, 2))
        .or_else(|| linea.find('→').map(|i| (i, '→'.len_utf8())))
}

/// Pista en español de por qué el autómata rechazó la línea.
fn explicar(linea: &str) -> String {
    let Some((i, largo)) = buscar_flecha(linea) else {
        return "no hay flecha (-> o →) entre la cabeza y los cuerpos".into();
    };
    let cabeza = linea[..i].trim();
    let derecha = linea[i + largo..].trim();

    match cabeza.chars().count() {
        0 => return "falta el no-terminal antes de la flecha".into(),
        1 if cabeza.chars().all(es_variable) => {}
        _ => return format!("la cabeza '{cabeza}' tiene que ser una sola letra MAYÚSCULA"),
    }
    if derecha.is_empty() {
        return "no hay nada después de la flecha (para vacío escriba ε)".into();
    }
    for alternativa in derecha.split('|').map(str::trim) {
        if alternativa.is_empty() {
            return "hay un | que no tiene cuerpo a uno de sus lados".into();
        }
        if alternativa.contains(VACIA) && alternativa != "ε" {
            return format!("en '{alternativa}' la ε debe ir sola");
        }
        if let Some(raro) = alternativa.chars().find(|c| !c.is_ascii_alphanumeric() && *c != VACIA) {
            return format!("el carácter '{raro}' no es un terminal ni un no-terminal");
        }
    }
    "no sigue la forma  A -> cuerpo | cuerpo ...".into()
}

pub fn leer(ruta: &str, afn: &Afn) -> Result<Lectura, Fallo> {
    let contenido = fs::read_to_string(ruta).map_err(|_| Fallo::NoSeAbre(ruta.to_string()))?;
    let contenido = contenido.trim_start_matches('\u{feff}');

    let mut buenas = Vec::new();
    let mut gramatica: Option<Gramatica> = None;

    for (i, cruda) in contenido.lines().enumerate() {
        let linea = cruda.trim_end();
        if linea.trim().is_empty() {
            continue;
        }

        let columna = match afn.evaluar(linea) {
            Veredicto::Acepta => None,
            Veredicto::TrabaEn(pos) => Some(pos),
            Veredicto::Incompleta => Some(linea.chars().count()),
        };
        if let Some(columna) = columna {
            let mala = LineaMala {
                numero: i + 1,
                texto: linea.to_string(),
                columna,
                motivo: explicar(linea),
            };
            return Err(Fallo::Linea { buenas, mala });
        }
        buenas.push((i + 1, linea.trim().to_string()));

        // La línea ya es válida: se separa en cabeza y cuerpos
        let (pos, largo) = buscar_flecha(linea).unwrap();
        let cabeza = linea[..pos].trim().chars().next().unwrap();
        let g = gramatica.get_or_insert_with(|| Gramatica::nueva(cabeza));
        for alternativa in linea[pos + largo..].split('|').map(str::trim) {
            let cuerpo = if alternativa == "ε" { Vec::new() } else { alternativa.chars().collect() };
            g.agregar(cabeza, cuerpo);
        }
    }

    let gramatica = gramatica.ok_or(Fallo::SinProducciones)?;

    let mut sin_reglas = Vec::new();
    for (_, cuerpos) in &gramatica.reglas {
        for &s in cuerpos.iter().flatten() {
            if es_variable(s) && !gramatica.tiene_reglas(s) && !sin_reglas.contains(&s) {
                sin_reglas.push(s);
            }
        }
    }

    Ok(Lectura { gramatica, buenas, sin_reglas })
}
