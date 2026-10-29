# Planeta Domo Espacial: integración técnica

Este domo es **el planeta de Regular Show (Un Show Más) del sistema solar**. No es una nave pilotable ni el escenario implementado del minijuego; los motores forman parte del modelo visual del planeta.

## Fuente preservada

Archivo: [fuente editable](../../assets/models/planets/regularshow/regularshow_space_dome_planet.blend). La preparación verificó una firma SHA-256 geométrica antes y después de normalizar la raíz y las rutas; no cambió vértices, topología, UV, asignaciones de material ni transformaciones del modelo aprobado.

Firma: `42baa95fb7a728d84f02a373baa0e8a518e49e810cac5ec2ee245c27aa0857e0`.

| Medida | Fuente | Cercano | Lejano |
| --- | ---: | ---: | ---: |
| Objetos de malla | 56 | 56 | 56 |
| Vértices Blender | 299,702 | 278,640 | 88,694 |
| Triángulos | 511,318 | 505,230 | 146,297 |
| Triángulos nulos/despreciables | 5871 | 0 | 0 |

Se usan conteos geométricos, no mediciones de FPS ni draw calls. El GLB puede separar vértices en bordes y UV o dividir primitivas por material. La versión lejana reduce 71.0 % de los triángulos cercanos.

## Transformaciones y alcance

Raíz: `RegularShow_Planet_ROOT`. Fuente con raíz unitaria y pivote original. La copia de juego usa factor 2.2 horneado, mantiene origen y proporciones y excluye cámaras, luces, fondo de estrellas y objetos ajenos al planeta. No se cambió la escala de los planetas de otros integrantes.

## Geometría de exportación

Se trianguló en copias, soldando coincidencias dentro de 0.00001 unidades de juego y retirando caras de área menor que 0.0000000001. Se conservaron las capas UV por esquina y las ranuras de material. El LOD usa simplificación por componente; sus siluetas y detalle pequeño deben revisarse según la distancia real de cámara.

## Materiales y portabilidad

Colores constantes sin imágenes externas. El shader completo del vidrio permanece en la fuente Blender. GLB/MTL usan transparencia alfa simplificada: vidrio 0.09, retícula 0.5, doble cara. Se debe comprobar ordenamiento y profundidad en el motor final.

Los GLB cercanos/lejanos son autónomos. Las imágenes externas de autoría o del OBJ se organizan bajo `assets/textures/planets/regularshow/`. Los materiales de Blender y la iluminación de presentación no equivalen a shaders del motor. El aspecto de la fuente sigue siendo la referencia artística.

## Pruebas realizadas

- Fuente reabierta y guardada en Blender 5.2.3 LTS con firma geométrica conservada.
- Archivos GLB válidos, sin URI externa de buffers o imágenes.
- Ambos GLB reimportados en Blender, con 505,230 y 146,297 triángulos, respectivamente.
- Error máximo de límites tras reimportar: 0.00000000 unidades.
- Cero triángulos nulos en las reimportaciones.
- Vistas cercano/lejano generadas para comparación.

## Pendientes del juego

El equipo todavía no ha elegido motor. Falta probar rendimiento en el dispositivo objetivo, seleccionar distancias de LOD, implementar carga/órbitas/rotación/proximidad y preparar colisiones simples. No se deben usar las mallas de alto detalle como colisionadores por defecto. Las transparencias requieren comprobar ordenamiento y profundidad en el destino.

Se adopta GLB como **formato de intercambio verificable**, sin imponer un motor ni cambiar los planes de gameplay de xPat95. Se conserva OBJ/MTL como alternativa. No se implementaron minijuegos, naves ni la estación espacial compartida.
