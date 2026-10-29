# Generación de End Ship

El archivo terminado se abre directamente desde `assets/models/ships/minecraft/end_ship.blend`. Estos scripts documentan su construcción; no son dependencias de ejecución del juego.

1. `inspect_source.py`: Python con Pillow. Lee la plantilla NBT de la instalación local Minecraft Java 1.21.11 y escribe los bloques, procedencia SHA-256 e inspección de las siete texturas en `docs/references/minecraft/end_ship/`.
2. `build_end_ship.py`: ejecutar con `blender --background --python tools/blender/minecraft/end_ship/build_end_ship.py`. Lee los datos de referencia y los modelos originales del JAR para resolver las orientaciones y formas de escaleras y slabs. Crea un archivo nuevo, empaqueta las texturas y genera tres renders. Se detiene si `end_ship.blend` ya existe para evitar sobrescribirlo accidentalmente.
3. `finish_and_verify.py`: procedimiento de acabado aplicado a esta entrega. **Modifica el .blend existente**: ajusta las UV del atlas de End Rods, reabre el archivo, genera las vistas de revisión, comprueba materiales, UV, parentesco, sombreado plano y ausencia de animaciones, y guarda el resultado. No es necesario ejecutarlo sobre la entrega terminada.

Las rutas del repositorio se resuelven desde la ubicación de cada script. La única fuente externa local es `%USERPROFILE%/AppData/Roaming/.minecraft/versions/1.21.11/1.21.11.jar`; adaptar `SOURCE` si se necesita regenerar en otra computadora. No se descarga contenido. El archivo final es autónomo y no necesita Minecraft instalado.
