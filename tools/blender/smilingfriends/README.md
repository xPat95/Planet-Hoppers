# Generadores del planeta Smiling Friends

Ambos scripts construyen su escena desde cero, guardan el `.blend` y renderizan capturas de revisión. Usan una semilla fija: misma semilla, mismo resultado. Solo usan módulos incluidos en Blender (`bmesh`, `numpy`).

| Script | Genera |
| --- | --- |
| `build_forest_demon.py` | `assets/models/characters/smilingfriends/forest_demon.blend` y `docs/references/smilingfriends/forest_demon_*.png`. |
| `build_smilingfriends_planet.py` | `assets/models/planets/smilingfriends/smilingfriends_planet.blend` y `docs/references/smilingfriends/smilingfriends_v01_*.png`. Importa el Forest Demon, así que se ejecuta después. |

Desde la raíz del repositorio, en PowerShell:

```powershell
& "C:\Program Files\Blender Foundation\Blender 5.2\blender.exe" -b --factory-startup --python tools/blender/smilingfriends/build_forest_demon.py
& "C:\Program Files\Blender Foundation\Blender 5.2\blender.exe" -b --factory-startup --python tools/blender/smilingfriends/build_smilingfriends_planet.py
```

Variables opcionales: `SF_NO_RENDER=1` guarda sin renderizar; `SF_OUTLINE=0` omite el contorno de caricatura.

Las posiciones de la distribución usan coordenadas de píxel del [boceto](../../../docs/references/smilingfriends/smilingfriends_layout_sketch.png) (función `sk`). Al terminar, el script imprime los triángulos por colección y avisa (`WARNING`) si algo choca.
