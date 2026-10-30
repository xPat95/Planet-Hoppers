# Autoría de la nave Butters

Scripts exclusivos de esta nave, sin cambios al código del juego ni a otras naves o planetas. Las rutas se resuelven desde la ubicación de los scripts, independientemente del directorio de trabajo.

Desde la raíz del repositorio, con Blender en PATH:

```powershell
blender -b --threads 6 --python-exit-code 1 --python tools/blender/southpark/ships/butters/build_butters_ship.py
blender -b assets/models/ships/southpark/butters/southpark_butters_ship.blend --threads 6 --python-exit-code 1 --python tools/blender/southpark/ships/butters/capture_butters_ship.py
blender -b assets/models/ships/southpark/butters/southpark_butters_ship.blend --threads 4 --python-exit-code 1 --python tools/blender/southpark/ships/butters/prepare_butters_ship.py
```

El generador reemplaza la fuente de Butters: conservar una copia si se ha editado a mano. El capturador genera cuatro PNG sin guardar cambios en el Blender. El preparador trabaja sobre copias evaluadas en un proceso desechable, agrupa por material, triangula, elimina caras degeneradas, exporta el GLB y comprueba su reimportación y la huella del archivo fuente. No ejecutarlo en la sesión interactiva del usuario: vacía la escena del proceso durante la verificación.

La instalación Alpha 5.4 usada para esta entrega no incluye algunos módulos del exportador de animación glTF. El preparador detecta el módulo ausente y evita cargar el subsistema de animación, exclusivamente en ese proceso. Si se intentara usar esa función, falla explícitamente. La exportación tiene animaciones desactivadas; no se modifica ningún archivo de la instalación.

La auditoría se escribe en `docs/references/southpark/ships/butters/integration_audit.json`. La textura de cartón es una imagen propia generada de forma determinista, con UV y empaquetada en el Blender.
