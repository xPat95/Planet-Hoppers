# Generador del planeta Halloween

El script construye la escena desde cero, guarda el `.blend` y renderiza capturas de revisión. Usa una semilla fija (`SEED`): misma semilla, mismo resultado. Solo usa módulos incluidos en Blender (`bmesh`, `numpy`).

| Script | Genera |
| --- | --- |
| `build_halloween_planet.py` | `assets/models/planets/halloween/halloween_planet.blend` y `docs/references/halloween/halloween_v01_*.png`. |

Desde la raíz del repositorio, en PowerShell:

```powershell
& "C:\Program Files\Blender Foundation\Blender 5.2\blender.exe" -b --factory-startup --python tools/blender/halloween/build_halloween_planet.py
```

Variable opcional: `HW_NO_RENDER=1` guarda sin renderizar.

La forma es una función de distancia con proporciones de cráneo real (1 unidad = 10 cm antes de escalar por `R`): `body` (cráneo), `holes` (órbitas, nariz y foramen magnum), `jaw` (mandíbula, girada `JAW_OPEN` sobre `JAW_PIVOT`) y `teeth` (arcos de adultos). `planet` vacía el cráneo en una cáscara de grosor `SHELL` y la malla sale de una rejilla de celda `H` (surface nets), que luego se simplifica a `TARGET_TRIS`. Al terminar, el script imprime los triángulos.
