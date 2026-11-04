# Generador del planeta Halloween

Los scripts construyen su escena desde cero, guardan el `.blend` y renderizan capturas de revisión. Usan una semilla fija (`SEED`): misma semilla, mismo resultado. Solo usan módulos incluidos en Blender (`bmesh`, `numpy`).

| Script | Genera |
| --- | --- |
| `build_witch.py` | `assets/models/characters/halloween/witch.blend` (un modelo por pose: `HW_Witch_Solo_A`, `Solo_B`, `Solo_C`, `Pair`, `Crawl`, `Climb`, `Peek`, `Sit`, `SitHappy`, `Ride`) y `docs/references/halloween/witches_*.png`. |
| `build_devil.py` | `assets/models/characters/halloween/devil.blend` (`HW_Devil_ArmsUp`, `Cactus`, `Point`, `Sway`) y `docs/references/halloween/devils_*.png`. |
| `build_halloween_planet.py` | `assets/models/planets/halloween/halloween_planet.blend` y `docs/references/halloween/halloween_v01_*.png`. Importa las brujas y los diablitos, así que se ejecuta después. |
| `build_witch_broom.py` | `assets/models/ships/halloween/witch_broom.blend` (la nave: la bruja `Ride` en su escoba, con la estela de estrellas) y `docs/references/halloween/broom_*.png`. Importa la bruja, así que va después de `build_witch.py`; luego se exporta con `export_solar_models.py -- halloween_ship`. |

Desde la raíz del repositorio, en PowerShell:

```powershell
& "C:\Program Files\Blender Foundation\Blender 5.2\blender.exe" -b --factory-startup --python tools/blender/halloween/build_witch.py
& "C:\Program Files\Blender Foundation\Blender 5.2\blender.exe" -b --factory-startup --python tools/blender/halloween/build_devil.py
& "C:\Program Files\Blender Foundation\Blender 5.2\blender.exe" -b --factory-startup --python tools/blender/halloween/build_halloween_planet.py
& "C:\Program Files\Blender Foundation\Blender 5.2\blender.exe" -b --factory-startup --python tools/blender/halloween/build_witch_broom.py
```

Variable opcional: `HW_NO_RENDER=1` guarda sin renderizar.

La forma es una función de distancia con proporciones de cráneo real (1 unidad = 10 cm antes de escalar por `R`): `body` (cráneo), `holes` (órbitas, nariz y foramen magnum), `jaw` (mandíbula, girada `JAW_OPEN` sobre `JAW_PIVOT`) y `teeth` (arcos de adultos). `planet` vacía el cráneo en una cáscara de grosor `SHELL` y la malla sale de una rejilla de celda `H` (surface nets), que luego se simplifica a `TARGET_TRIS`; la versión lejana (`FAR_TRIS`) va en la colección oculta `LOD_Far`. Al terminar, el script imprime los triángulos.

Las brujas se arman sobre un esqueleto simple: cada pose de `POSES` en `build_witch.py` es un juego de ángulos (cadera, torso, cabeza, piernas y brazos). Con `smooth=True` (solo la de la escoba, que se ve de cerca) el cuerpo se hace de una sola pieza con `SmoothBody` de `figure.py`: los huesos y elipsoides se unen en suave en una función de distancia, la malla sale por surface nets, se corta por planos en cada borde de color (rayas, sisas, escote) y cada triángulo que queda entre dos piezas se parte por la línea donde las dos están igual de cerca, así los bordes de color no salen serruchados. En el planeta, `LAYOUTS` define qué va en cada órbita; `floor_spot` y `brow_spot` buscan con rayos el piso y el borde de arriba de cada órbita.

`figure.py` tiene las piezas compartidas por las brujas y los diablitos: el material plano, el `Builder` de tubos y elipsoides y los giros. La grieta de la frente se define en `FOREHEAD_PATHS` (vista de frente); los diablitos bailan repartidos parejo por todas las grietas (`DEVIL_COUNT`), con notas musicales alrededor.
