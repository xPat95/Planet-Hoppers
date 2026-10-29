# Planeta Estrella de la Muerte: integración técnica

Esta Estrella de la Muerte es **el planeta de Star Wars del sistema solar**. No sustituye la estación espacial compartida del hangar, no es una nave y no implementa el minijuego tipo Galaga.

## Fuente preservada

Archivo: [fuente editable](../../assets/models/planets/starwars/starwars_death_star_planet.blend). La preparación verificó una firma SHA-256 geométrica antes y después de normalizar la raíz y las rutas; no cambió vértices, topología, UV, asignaciones de material ni transformaciones del modelo aprobado.

Firma: `f287177fca902f2c844f7a7742a19c5a6cdc1722dbd427cbe856bf7adfccb970`.

| Medida | Fuente | Cercano | Lejano |
| --- | ---: | ---: | ---: |
| Objetos de malla | 9 | 9 | 7 |
| Vértices Blender | 1,014,252 | 315,687 | 40,687 |
| Triángulos | 507,082 | 506,058 | 69,670 |
| Triángulos nulos/despreciables | 1024 | 0 | 0 |

Se usan conteos geométricos, no mediciones de FPS ni draw calls. El GLB puede separar vértices en bordes y UV o dividir primitivas por material. La versión lejana reduce 86.2 % de los triángulos cercanos.

## Transformaciones y alcance

Raíz: `StarWars_Planet_ROOT`. Fuente con raíz unitaria y pivote original. La copia de juego usa factor 2.2 horneado, mantiene origen y proporciones y excluye cámaras, luces, fondo de estrellas y objetos ajenos al planeta. No se cambió la escala de los planetas de otros integrantes.

## Geometría de exportación

Se trianguló en copias, soldando coincidencias dentro de 0.00001 unidades de juego y retirando caras de área menor que 0.0000000001. Se conservaron las capas UV por esquina y las ranuras de material. El LOD usa simplificación por componente; sus siluetas y detalle pequeño deben revisarse según la distancia real de cámara.

## Materiales y portabilidad

Los dos atlas de autoría están empaquetados dentro del Blender. Las exportaciones usan doce mapas sRGB de blindaje y una normal tangente aproximada calculada a partir del microrelieve; el OBJ conserva el mapa de altura. Los otros materiales son colores constantes.

Los GLB cercanos/lejanos son autónomos. Las imágenes externas de autoría o del OBJ se organizan bajo `assets/textures/planets/starwars/`. Los materiales de Blender y la iluminación de presentación no equivalen a shaders del motor. El aspecto de la fuente sigue siendo la referencia artística.

## Pruebas realizadas

- Fuente reabierta y guardada en Blender 5.2.3 LTS con firma geométrica conservada.
- Archivos GLB válidos, sin URI externa de buffers o imágenes.
- Ambos GLB reimportados en Blender, con 506,058 y 69,670 triángulos, respectivamente.
- Error máximo de límites tras reimportar: 0.00000000 unidades.
- Cero triángulos nulos en las reimportaciones.
- Vistas cercano/lejano generadas para comparación.

## Pendientes del juego

El equipo todavía no ha elegido motor. Falta probar rendimiento en el dispositivo objetivo, seleccionar distancias de LOD, implementar carga/órbitas/rotación/proximidad y preparar colisiones simples. No se deben usar las mallas de alto detalle como colisionadores por defecto. Las transparencias requieren comprobar ordenamiento y profundidad en el destino.

Se adopta GLB como **formato de intercambio verificable**, sin imponer un motor ni cambiar los planes de gameplay de xPat95. Se conserva OBJ/MTL como alternativa. No se implementaron minijuegos, naves ni la estación espacial compartida.
