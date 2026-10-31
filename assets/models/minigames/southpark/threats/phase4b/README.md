# Recursos derivados de fase 4B

`southpark_threats_4b.blend` contiene cinco escenas editables, independientes del pueblo y de los recursos de fase 4A. Metros, Z arriba en Blender; exportación GLB con Y arriba, sin cambio de escala. El origen de cada modelo está a nivel del suelo. El soporte usa como origen el centro de impacto del piano.

| Exportación | Objetos | Triángulos | Dimensiones aproximadas (X, altura, Z) |
| --- | ---: | ---: | --- |
| `piano.glb` | 47 | 740 | 3.98 × 1.81 × 2.40 m |
| `piano_support.glb` | 5 | 124 | 0.85 × 9.40 × 2.93 m |
| `piano_lock.glb` | 5 | 92 | 0.76 × 1.70 × 0.55 m |
| `snowman.glb` | 25 | 2160 | 2.09 × 3.23 × 1.48 m |
| `heater.glb` | 10 | 120 | 0.90 × 1.37 × 0.70 m |

Paleta plana `SPT_4B_*`, sin texturas externas, rigging ni dependencias de otros modelos. El calefactor se instancia tres veces desde el mismo recurso. Rust dibuja sus indicadores y panel de calor para cambiar el estado sin recargar materiales. El muñeco mira hacia -Z local en GLB. Teclado del piano hacia -X; paneles hacia -Z. Las colisiones simplificadas se definen en el director, separadas de las mallas.

Generar desde la raíz (reemplazar `blender` por su ruta local si corresponde):

```powershell
blender --background --python-exit-code 1 --python tools/blender/southpark/build_threat_assets_4b.py
```

El generador inicia escenas nuevas y escribe únicamente esta carpeta. No abre el planeta, el pueblo, las naves, el PNG ni el archivo de amenazas 4A. `asset-audit.json` registra recuentos y cajas de Blender; el cargador Rust comprueba los triángulos y la conversión de coordenadas al cargar los cinco GLB.

Validar sin guardar modelos:

```powershell
blender --background --python-exit-code 1 --python tools/blender/southpark/validate_threat_assets_4b.py
```

Documentación del juego: [fase 4B](../../../../../../docs/minigames/southpark-phase4b.md).
