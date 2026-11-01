# Herramientas de la Ciudadela

- `crear_ciudadela.py`: reconstruye la fuente procedural aprobada desde un archivo nuevo de Blender. Escribe en `assets/models/environment/spaceStation/` y `docs/references/spaceStation/`.
- `renderizar_ciudadela.py`: genera las cuatro vistas a partir de la fuente abierta.
- `prepare_citadel_asset.py`: actualiza los metadatos de la copia integrada, conserva su geometria, genera GLB cercano/lejano y OBJ/MTL, y reimporta los GLB para comprobar jerarquia, cinco brazos, triangulos, limites y dependencias.
- `validate_citadel_package.py`: valida enlaces, sintaxis, hashes, estructura GLB y OBJ/MTL sin Blender.

Desde la raiz del repositorio:

```powershell
blender -b --python tools/blender/spaceStation/prepare_citadel_asset.py
blender -b assets/models/environment/spaceStation/citadel_space_station.blend --python tools/blender/spaceStation/renderizar_ciudadela.py
python tools/blender/spaceStation/validate_citadel_package.py
```

El exportador usa [asset.json](../../../assets/models/environment/spaceStation/asset.json). La geometria de exportacion se procesa en memoria; no se guardan los LOD sobre la fuente. El original de ProyectoFinal no se necesita para ejecutar estas herramientas.
