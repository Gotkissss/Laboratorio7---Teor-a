//! Motor de expresiones regulares hecho a mano (idea del Proyecto 1):
//!   1. Se parte el patrón en piezas (literales, clases, operadores).
//!   2. Se agrega la concatenación explícita y se pasa a postfix (Shunting-yard).
//!   3. Con el postfix se arma un AFN usando la construcción de Thompson.
//!   4. Se simula el AFN sobre la línea con cerraduras-ε.
//!
//! Soporta: literales, escapes (\|), clases [A-Z0-9], agrupación ( ),
//! unión |, cerradura de Kleene *, cerradura positiva + y opcional ?.

#[derive(Clone, Debug)]
enum Pieza {
    Simbolo(Vec<(char, char)>), // un carácter o una clase, guardados como rangos
    Union,
    Concat,
    Kleene,
    Positiva,
    Opcional,
    Abre,
    Cierra,
}

impl Pieza {
    fn es_operando_izq(&self) -> bool {
        matches!(self, Pieza::Simbolo(_) | Pieza::Cierra | Pieza::Kleene | Pieza::Positiva | Pieza::Opcional)
    }
    fn es_operando_der(&self) -> bool {
        matches!(self, Pieza::Simbolo(_) | Pieza::Abre)
    }
    fn precedencia(&self) -> u8 {
        match self {
            Pieza::Union => 1,
            Pieza::Concat => 2,
            _ => 0,
        }
    }
}

fn separar(patron: &str) -> Result<Vec<Pieza>, String> {
    let mut piezas = Vec::new();
    let mut it = patron.chars().peekable();

    while let Some(c) = it.next() {
        let pieza = match c {
            '\\' => {
                let sig = it.next().ok_or("el patrón termina en '\\'")?;
                Pieza::Simbolo(vec![(sig, sig)])
            }
            '[' => {
                let mut rangos = Vec::new();
                loop {
                    let desde = it.next().ok_or("falta cerrar ']'")?;
                    if desde == ']' {
                        break;
                    }
                    if it.peek() == Some(&'-') {
                        it.next();
                        let hasta = it.next().ok_or("rango incompleto en la clase")?;
                        rangos.push((desde, hasta));
                    } else {
                        rangos.push((desde, desde));
                    }
                }
                Pieza::Simbolo(rangos)
            }
            '|' => Pieza::Union,
            '*' => Pieza::Kleene,
            '+' => Pieza::Positiva,
            '?' => Pieza::Opcional,
            '(' => Pieza::Abre,
            ')' => Pieza::Cierra,
            otro => Pieza::Simbolo(vec![(otro, otro)]),
        };

        // Concatenación implícita entre dos piezas seguidas: "ab" -> a · b
        if let Some(anterior) = piezas.last() {
            if Pieza::es_operando_izq(anterior) && pieza.es_operando_der() {
                piezas.push(Pieza::Concat);
            }
        }
        piezas.push(pieza);
    }
    Ok(piezas)
}

fn a_postfix(piezas: Vec<Pieza>) -> Result<Vec<Pieza>, String> {
    let mut salida = Vec::new();
    let mut pila: Vec<Pieza> = Vec::new();

    for p in piezas {
        match p {
            // Los unarios son postfijos y de mayor precedencia: salen directo
            Pieza::Simbolo(_) | Pieza::Kleene | Pieza::Positiva | Pieza::Opcional => salida.push(p),
            Pieza::Abre => pila.push(p),
            Pieza::Cierra => loop {
                match pila.pop() {
                    Some(Pieza::Abre) => break,
                    Some(op) => salida.push(op),
                    None => return Err("paréntesis ')' sin abrir".into()),
                }
            },
            Pieza::Union | Pieza::Concat => {
                while let Some(tope) = pila.last() {
                    if !matches!(tope, Pieza::Abre) && tope.precedencia() >= p.precedencia() {
                        salida.push(pila.pop().unwrap());
                    } else {
                        break;
                    }
                }
                pila.push(p);
            }
        }
    }
    while let Some(op) = pila.pop() {
        if matches!(op, Pieza::Abre) {
            return Err("paréntesis '(' sin cerrar".into());
        }
        salida.push(op);
    }
    Ok(salida)
}

/// Transición del AFN: `None` es una transición-ε.
type Arista = (Option<Vec<(char, char)>>, usize);

pub struct Afn {
    aristas: Vec<Vec<Arista>>,
    inicio: usize,
    aceptacion: usize,
}

pub enum Veredicto {
    Acepta,
    /// El autómata se quedó sin estados al leer el carácter de esta posición.
    TrabaEn(usize),
    /// Se leyó toda la línea pero no se llegó a un estado de aceptación.
    Incompleta,
}

impl Afn {
    fn estado_nuevo(&mut self) -> usize {
        self.aristas.push(Vec::new());
        self.aristas.len() - 1
    }

    fn unir(&mut self, de: usize, a: usize, con: Option<Vec<(char, char)>>) {
        self.aristas[de].push((con, a));
    }

    /// Construcción de Thompson a partir del postfix.
    fn thompson(postfix: Vec<Pieza>) -> Result<Afn, String> {
        let mut afn = Afn { aristas: Vec::new(), inicio: 0, aceptacion: 0 };
        let mut pila: Vec<(usize, usize)> = Vec::new();
        let falta = || "faltan operandos en el patrón".to_string();

        for p in postfix {
            let fragmento = match p {
                Pieza::Simbolo(rangos) => {
                    let (i, f) = (afn.estado_nuevo(), afn.estado_nuevo());
                    afn.unir(i, f, Some(rangos));
                    (i, f)
                }
                Pieza::Concat => {
                    let b = pila.pop().ok_or_else(falta)?;
                    let a = pila.pop().ok_or_else(falta)?;
                    afn.unir(a.1, b.0, None);
                    (a.0, b.1)
                }
                Pieza::Union => {
                    let b = pila.pop().ok_or_else(falta)?;
                    let a = pila.pop().ok_or_else(falta)?;
                    let (i, f) = (afn.estado_nuevo(), afn.estado_nuevo());
                    afn.unir(i, a.0, None);
                    afn.unir(i, b.0, None);
                    afn.unir(a.1, f, None);
                    afn.unir(b.1, f, None);
                    (i, f)
                }
                Pieza::Kleene | Pieza::Positiva | Pieza::Opcional => {
                    let a = pila.pop().ok_or_else(falta)?;
                    let (i, f) = (afn.estado_nuevo(), afn.estado_nuevo());
                    afn.unir(i, a.0, None);
                    afn.unir(a.1, f, None);
                    if !matches!(p, Pieza::Positiva) {
                        afn.unir(i, f, None); // puede no aparecer
                    }
                    if !matches!(p, Pieza::Opcional) {
                        afn.unir(a.1, a.0, None); // puede repetirse
                    }
                    (i, f)
                }
                Pieza::Abre | Pieza::Cierra => unreachable!(),
            };
            pila.push(fragmento);
        }

        let (i, f) = pila.pop().ok_or("el patrón está vacío")?;
        if !pila.is_empty() {
            return Err("el patrón tiene piezas sin operador".into());
        }
        afn.inicio = i;
        afn.aceptacion = f;
        Ok(afn)
    }

    fn cerradura(&self, mut estados: Vec<usize>) -> Vec<bool> {
        let mut dentro = vec![false; self.aristas.len()];
        for &e in &estados {
            dentro[e] = true;
        }
        while let Some(e) = estados.pop() {
            for (con, destino) in &self.aristas[e] {
                if con.is_none() && !dentro[*destino] {
                    dentro[*destino] = true;
                    estados.push(*destino);
                }
            }
        }
        dentro
    }

    pub fn evaluar(&self, texto: &str) -> Veredicto {
        let mut actuales = self.cerradura(vec![self.inicio]);

        for (pos, c) in texto.chars().enumerate() {
            let mut siguientes = Vec::new();
            for (e, _) in actuales.iter().enumerate().filter(|(_, &activo)| activo) {
                for (con, destino) in &self.aristas[e] {
                    if let Some(rangos) = con {
                        if rangos.iter().any(|&(a, b)| a <= c && c <= b) {
                            siguientes.push(*destino);
                        }
                    }
                }
            }
            if siguientes.is_empty() {
                return Veredicto::TrabaEn(pos);
            }
            actuales = self.cerradura(siguientes);
        }

        if actuales[self.aceptacion] {
            Veredicto::Acepta
        } else {
            Veredicto::Incompleta
        }
    }

    pub fn cantidad_estados(&self) -> usize {
        self.aristas.len()
    }
}

pub fn compilar(patron: &str) -> Result<Afn, String> {
    Afn::thompson(a_postfix(separar(patron)?)?)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn acepta(patron: &str, texto: &str) -> bool {
        matches!(compilar(patron).unwrap().evaluar(texto), Veredicto::Acepta)
    }

    #[test]
    fn operadores_basicos() {
        assert!(acepta("ab*", "abbb"));
        assert!(acepta("ab*", "a"));
        assert!(!acepta("ab+", "a"));
        assert!(acepta("(a|b)?c", "c"));
        assert!(acepta("[A-Z]\\|", "Q|"));
        assert!(!acepta("[A-Z]", "q"));
    }
}
