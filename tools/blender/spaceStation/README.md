# Herramientas de la Ciudadela

## Fase 1 artística: reconstrucción desde la fuente real

`rebuild_hangar_phase1.py` abre la estación aprobada, reutiliza sus materiales y genera archivos independientes `citadel_hangar_phase1.blend` y `exports/citadel_hangar_phase1.glb`. Construye la arquitectura sin exportar plataforma ni naves. `validate_hangar_phase1.py`, ejecutado con Blender, vuelve a abrir los dos `.blend`, inspecciona transformaciones/materiales y reimporta el GLB para comprobar huecos, espacio libre y límites. [Renders, capturas y alcance](../../../docs/references/hangar/phase1/README.md).

```powershell
blender -b --factory-startup --python tools/blender/spaceStation/rebuild_hangar_phase1.py
blender -b --factory-startup --python tools/blender/spaceStation/validate_hangar_phase1.py
```

## Interior complementario del hangar

`inspect_hangar_reference.py` abre la fuente aprobada y genera inspección y renders sin guardarla. `build_hangar.py` reutiliza sus materiales y construye el módulo independiente `citadel_hangar_interior.blend` y su GLB, sin incluir la plataforma de exhibición de Rust. `validate_hangar.py` verifica el GLB con Python y, ejecutado desde Blender, lo reimporta. [Evidencias y validaciones](../../../docs/references/hangar/README.md).

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
