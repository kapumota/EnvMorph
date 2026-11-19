# Carpeta campaign

Esta carpeta debe estar VACIA hasta el congelamiento. Los archivos que la guardia exige son:

- PROTOCOL_FREEZE.json   lo genera harness/g3c_freeze.py (hash de protocolo, pool, harness, factores y specs).
- decisions.json         lo genera `g3c_decide.py decide` a partir de las celdas S.
- decisions.sha256 y SEALED.json   los genera `g3c_decide.py seal`.

Sin PROTOCOL_FREEZE.json el harness rechaza ejecutar cualquier celda que no sea el preflight del baseline.
Sin SEALED.json coincidente con decisions.json el harness rechaza ejecutar celdas held-out.
Si existe alguna celda held-out ejecutada, `decide` aborta por posible fuga.
