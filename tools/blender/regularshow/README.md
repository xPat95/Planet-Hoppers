# Scripts del planeta domo espacial

Las rutas se calculan desde el repositorio, siguiendo la organización de los scripts de South Park de xPat95. No necesitan rutas absolutas de la computadora del autor.

## Preparación de las copias de juego

Desde la raíz del repositorio:

```powershell
blender -b --threads 6 --python-exit-code 1 --python tools/blender/regularshow/prepare_regularshow_planet.py
```

Este comando abre la fuente principal en `assets/models/planets/regularshow/`, conserva su geometría editable, normaliza metadatos/raíz y rutas, genera los archivos de `exports/`, renderiza las vistas cercano/lejano en `docs/references/regularshow/` y guarda una auditoría JSON tras reimportar ambos GLB.

No ejecutar el script para simplemente abrir o visualizar el `.blend`. No procesa archivos de otras franquicias.

## Autoría procedural

Los generadores y módulos originales se conservan juntos en esta carpeta. Guardan los modelos en `assets/models/planets/regularshow/` y los renders en `docs/references/regularshow/`. Los scripts auxiliares importan módulos vecinos mediante rutas relativas a `__file__`.

El exportador de integración es `prepare_regularshow_planet.py`: genera el OBJ limpio y los dos GLB verificados. Los exportadores históricos de ProyectoFinal se conservan con los originales, fuera del repositorio.

La regeneración procedural reemplaza su fuente: hacer una copia local antes de regenerar. La fuente principal aprobada y el script de preparación son la referencia de integración. No se copiaron `__pycache__` ni respaldos históricos al commit.
