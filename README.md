# Laboratorio 7 · Teoría de la Computación

Programa en **Rust** que carga gramáticas libres de contexto desde archivos de texto, valida cada línea con una **expresión regular convertida a AFN (Thompson)** y **elimina las producciones-ε** mostrando cada fase del algoritmo.

## Video

**Demostración: ** https://youtu.be/oXTBVqJYoXg

## Contenido

| Ruta | Qué es |
|------|--------|
| `src/main.rs` | Punto de entrada: recorre los archivos y muestra resultados |
| `src/automata.rs` | Motor de regex propio: Shunting-yard → postfix → AFN de Thompson → simulación |
| `src/lector.rs` | Lectura del archivo, validación línea por línea y armado de la gramática |
| `src/epsilon.rs` | Algoritmo para quitar producciones-ε (anulables + 2^m combinaciones) |
| `src/pantalla.rs` | Formato y colores de la salida |
| `entradas/g1.txt`, `g2.txt`, `g3.txt` | Gramáticas del Problema 2 |
| `entradas/con_errores/` | Archivos con errores para probar la validación |

## Requisitos

Rust con `cargo` (ya viene con la instalación de Rust). No usa librerías externas, así que no descarga nada.

```
cargo --version
```

## Ejecución

Desde la carpeta del proyecto, en la terminal de VS Code:

```
cargo run -q -- entradas/g1.txt
cargo run -q -- entradas/g2.txt
cargo run -q -- entradas/g3.txt
```

Varias a la vez:

```
cargo run -q -- entradas/g1.txt entradas/g2.txt entradas/g3.txt
```

Validación con errores (el programa se detiene e indica la columna donde el autómata falla):

```
cargo run -q -- entradas/con_errores/falta_flecha.txt
cargo run -q -- entradas/con_errores/cabeza_terminal.txt
cargo run -q -- entradas/con_errores/or_sin_cuerpo.txt
cargo run -q -- entradas/con_errores/caracter_raro.txt
```

Pruebas del motor de regex: `cargo test`

## Formato de las gramáticas

```
S -> 0A0 | 1B1 | BB
C -> S | ε
```

- Mayúscula = no-terminal; minúscula o dígito = terminal; `ε` = cadena vacía.
- Flecha `->` o `→`. Alternativas separadas por `|`. La primera cabeza es el símbolo inicial.

Expresión regular usada:

```
 *[A-Z] *(->|→) *([a-zA-Z0-9]+|ε)( *\| *([a-zA-Z0-9]+|ε))* *
```

## Algoritmo

1. **Fase 1 – Anulables:** se parte de N = {} y por rondas se agrega A si tiene `A → ε` o un cuerpo con todos sus símbolos en N, hasta que N deja de crecer.
2. **Fase 2 – Producciones anulables:** se listan las reglas cuyo cuerpo está completamente en N (o es ε).
3. **Fase 3 – 2^m combinaciones:** por cada regla con m anulables se recorren las máscaras de bits de 0 a 2^m − 1 (1 = quitar ese anulable). Se descartan ε, duplicados y `A → A`.
4. Se imprime la gramática final. Si el símbolo inicial es anulable, se indica que el resultado genera L(G) − {ε}.
