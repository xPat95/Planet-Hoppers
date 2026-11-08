# Minijuego de Halloween: modelos

Diseño: [plan del minijuego](../../../../docs/minigames/halloween-plan.md).

Cada personaje viene en varias poses con **la misma malla** (mismos vértices en el mismo orden), una pose por GLB: el juego las mezcla para que el movimiento sea suave. Origen en el piso.

| Archivo | Qué es | Triángulos |
| --- | --- | ---: |
| `witch_solo_a/b/c`, `witch_hop`, `witch_sway`, `witch_spin` | La bruja (la de la escoba: pelo azul azabache, flequillo, nariz, ojos de cuenta rojos) a pie, bailando. | 4 882 |
| `witch_cast` | La bruja lanzando un hechizo. | 4 882 |
| `devil_walk_a/b`, `devil_dance_a/b` | Diablo rojo alto como los fantasmas (~1.2 con los cuernos) y muy flaco, con la silueta de la referencia de Cami (`docs/references/halloween/devil_body_reference.png`): brazos que llegan a las rodillas, dedos larguísimos con garras, la caja de costillas y las clavículas marcadas bajo la piel. Patas de cabra con pezuñas, cuernos rojos enroscados y la cara tallada como una calabaza (`devil_face_reference.png`). Camina muy encorvado, con la cara tallada adelante y bajada, la cabeza ladeada y los brazos colgando. Soldado y con huesos. | 10 936 |
| `ghost_float_a/b`, `ghost_dance_a/b` | Fantasma: una persona de verdad (cabeza, cuello, hombros, brazos) debajo de una sábana fina llena de pliegues, de ~1.2 de alto. Dos agujeros redondos recortados en la tela con ojos bien abiertos, sin párpados, de pupila diminuta en un aro azul (mirada muerta), y un recorte con una sonrisa muy ancha llena de dientes. Soldado y con huesos como el demonio. **La sábana es de una sola capa: el juego tiene que dibujarlo sin descartar las caras de atrás (backface culling apagado).** | 15 587 |
| `demon_walk_a/b`, `demon_dance_a/b` | Demonio de parálisis del sueño: esqueleto negro demasiado alto, con cuatro brazos de hueso y garras, la caja de costillas como detalle principal, cuello largo y cara de calavera que grita (referencia de Cami en `docs/references/halloween/demon_face_reference.png`): cráneo pelado con la frente arrugada, cuencas enormes con un puntito blanco al fondo, nariz de calavera y la boca abierta en un óvalo, con colmillos arriba y abajo; mira hacia abajo a quien tiene adelante. Piernas dobladas para atrás. Es una sola malla soldada (los huesos se funden unos con otros) que se dobla con huesos para cada pose. | 12 762 |

## Edición

No editar a mano: la bruja sale de [build_witch.py](../../../../tools/blender/halloween/build_witch.py) (`GAME_POSES`, colección `Game` de `witch.blend`) y las criaturas de [build_game_creatures.py](../../../../tools/blender/halloween/build_game_creatures.py) (`creatures.blend`). Después se exportan con [export_solar_models.py](../../../../tools/blender/solarsystem/export_solar_models.py) (`halloween_game_witch_<pose>`, `halloween_game_<criatura>_<pose>`). Al exportar, la raíz de cada pose se pone en el origen: todas las poses comparten el piso y se pueden mezclar. Captura: `docs/references/halloween/game_creatures.png`.
