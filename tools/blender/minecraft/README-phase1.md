# Escenario explorable Overworld

Estos scripts son independientes de los generadores históricos del planeta y la End Ship.

Desde la raíz del repositorio, con Blender 5.4 Alpha o una versión compatible con su API:

```powershell
blender --background --factory-startup --python-exit-code 1 --python tools/blender/minecraft/build_explorable_village.py
blender --background --factory-startup --python-exit-code 1 --python tools/blender/minecraft/validate_explorable_village.py
```

El primero reconstruye únicamente `assets/models/minigames/minecraft/` y las capturas/inventario de `docs/references/minecraft/phase1/`. No ejecutar sobre ediciones manuales del derivado que se quieran conservar. La opción `-- --skip-renders` permite omitir renders durante ajustes técnicos; regenerar normalmente antes de entregar para actualizar las cinco capturas.

El segundo solo lee modelos originales y derivados, reabre el `.blend`, reimporta el GLB y escribe `validation.json`; falla con excepción ante un error. Requiere `protected_sources.json`, registro de hashes previo a esta entrega. No regenerar ese registro como forma de omitir diferencias en archivos protegidos.

No se requieren bibliotecas de Python externas a Blender. Se usa NumPy incluido con Blender para copias de texturas teñidas. El workaround de exportación estática para la distribución Alpha está contenido en el proceso y no modifica el exportador instalado.

Consultar [guía y contrato de fase 1](../../../docs/minigames/minecraft-phase1.md).
