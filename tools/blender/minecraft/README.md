# Herramientas Blender del planeta Minecraft

- `build_minecraft_planet.py`: generador histórico del blockout v01. Crea una escena nueva; no guarda automáticamente. No es necesario ejecutarlo para abrir el modelo actual.
- `texture_minecraft_planet_v02.py`: procedimiento histórico de texturizado sobre v01, con copia de seguridad previa. No ejecutar sobre la versión ya texturizada. Solo modifica materiales y UV y no guarda automáticamente el `.blend`.

Para utilizarlos manualmente, abrir el archivo correspondiente desde el editor de texto de Blender y ejecutarlo allí. El script de texturizado requiere `__file__` definido; usar el archivo abierto desde disco, no una copia de texto sin ubicación.

Las rutas de texturas y diagnósticos se resuelven desde la ubicación del script en el repositorio, sin una ruta fija de computadora. Las texturas están en `assets/textures/planets/minecraft/` y los diagnósticos se escriben en `docs/references/minecraft/diagnostics/`.

El modelo principal está en `assets/models/planets/minecraft/minecraft_planet_textured_v02.blend`. Sus scripts internos son copias de referencia con rutas relativas a esta carpeta. No se requiere ejecutar código ni instalar dependencias para visualizar el modelo.
