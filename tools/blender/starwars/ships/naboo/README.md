# Autoría de Caza Naboo N-1

Los generadores nativos usan rutas relativas al repositorio y conservan sus módulos auxiliares juntos. La fuente aprobada está en `assets/models/ships/starwars/naboo/naboo_n1_royal_ship.blend`. La regeneración reemplaza esa fuente; conservar una copia antes de reconstruir.

Para generar o verificar las copias de juego desde la raíz:

```powershell
blender -b --threads 6 --python-exit-code 1 --python tools/blender/ships/prepare_ship_asset.py -- --asset assets/models/ships/starwars/naboo/asset.json
```

La preparación no guarda ni modifica la fuente editable. OBJ/GLB, LOD y personajes proceden de copias evaluadas en memoria. El informe se escribe en `docs/references/starwars/ships/naboo/integration_audit.json`. La maquinaria, las caras y los rasgos de los personajes son geometría nativa, sin modelos descargados.
