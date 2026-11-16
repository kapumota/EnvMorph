# M3: lectura del replay retrospectivo P1-EB-01

**ESTADO:** comparación retrospectiva exploratoria; NO es confirmación independiente ni GO metodológico.

Los 12 workflows de 2 factores se extraen de solo 6 casos y 2 proyectos upstream.
Cada política empieza observando el baseline con dos repeticiones, con costo contabilizado.
La tabla exacta comparte las mismas consultas. F6-v2 tiene cobertura factual directa idéntica a esa tabla.
`model_supported` es inferencia condicional al modelo H; `model_errors` mide errores reales contra la referencia.
Las cinco semillas aleatorias se presentan por separado, nunca como 60 workflows independientes.

| Política | Semilla | Presupuesto/workflow | Directas/48 | Inferidas/48 | Erróneas | No resueltas |
|---|---:|---:|---:|---:|---:|---:|
| greedy_coverage | -1 | 2 | 12 | 0 | 0 | 36 |
| greedy_coverage | -1 | 4 | 24 | 0 | 0 | 24 |
| greedy_coverage | -1 | 6 | 36 | 0 | 0 | 12 |
| greedy_coverage | -1 | 8 | 48 | 0 | 0 | 0 |
| lexicographic | -1 | 2 | 12 | 0 | 0 | 36 |
| lexicographic | -1 | 4 | 24 | 0 | 0 | 24 |
| lexicographic | -1 | 6 | 36 | 0 | 0 | 12 |
| lexicographic | -1 | 8 | 48 | 0 | 0 | 0 |
| pairwise | -1 | 2 | 12 | 0 | 0 | 36 |
| pairwise | -1 | 4 | 24 | 0 | 0 | 24 |
| pairwise | -1 | 6 | 36 | 0 | 0 | 12 |
| pairwise | -1 | 8 | 48 | 0 | 0 | 0 |
| random | 0 | 2 | 12 | 0 | 0 | 36 |
| random | 0 | 4 | 24 | 3 | 0 | 21 |
| random | 0 | 6 | 36 | 4 | 0 | 8 |
| random | 0 | 8 | 48 | 0 | 0 | 0 |
| random | 1 | 2 | 12 | 0 | 0 | 36 |
| random | 1 | 4 | 24 | 2 | 0 | 22 |
| random | 1 | 6 | 36 | 2 | 0 | 10 |
| random | 1 | 8 | 48 | 0 | 0 | 0 |
| random | 2 | 2 | 12 | 0 | 0 | 36 |
| random | 2 | 4 | 24 | 1 | 0 | 23 |
| random | 2 | 6 | 36 | 2 | 0 | 10 |
| random | 2 | 8 | 48 | 0 | 0 | 0 |
| random | 3 | 2 | 12 | 0 | 0 | 36 |
| random | 3 | 4 | 24 | 4 | 0 | 20 |
| random | 3 | 6 | 36 | 3 | 0 | 9 |
| random | 3 | 8 | 48 | 0 | 0 | 0 |
| random | 4 | 2 | 12 | 0 | 0 | 36 |
| random | 4 | 4 | 24 | 3 | 0 | 21 |
| random | 4 | 6 | 36 | 3 | 0 | 9 |
| random | 4 | 8 | 48 | 0 | 0 | 0 |
| rust_suggested | -1 | 2 | 12 | 0 | 0 | 36 |
| rust_suggested | -1 | 4 | 24 | 0 | 0 | 24 |
| rust_suggested | -1 | 6 | 36 | 0 | 0 | 12 |
| rust_suggested | -1 | 8 | 48 | 0 | 0 | 0 |

**Advertencia:** pairwise se degenera en desempate lexicográfico con dos factores; no se interpreta como comparación de covering arrays.

El análisis de M3 sirve para detectar diferencias entre políticas y errores bajo este dominio pequeño.
No sirve para afirmar novedad, generalización entre proyectos ni superioridad Q1/Q2.
