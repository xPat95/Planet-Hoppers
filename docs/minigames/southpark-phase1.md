# SAVE KENNY! — fase 1: escenario explorable

## Inspección y alcance

El repositorio inspeccionado contiene modelos y documentación, pero `src/`, `config/` y `tests/` solo tenían archivos `.gitkeep`. No existen framework, dependencias del juego, ejecutable, rutas ni administrador de escenas que se puedan reutilizar. `docs/architecture.md` define una separación conceptual que se respeta. Tampoco existe todavía un sistema solar ejecutable sobre el que hacer pruebas de regresión.

Se inspeccionaron el principal `assets/models/planets/southpark/southpark_planet.blend`, sus generadores históricos, inventarios v1/v2/v3, revisión técnica y documentación. No había una exportación del planeta. El modelo tiene objetos separados, coordenadas `reference_xy`, jerarquías y 28 materiales Principled de colores planos, sin imágenes externas. La imagen preexistente bajo `assets/textures/minigames/South Park/` quedó sin utilizar ni modificar: los personajes están fuera de esta fase.

Blender 5.4.0 Alpha está disponible. Se utilizó un proceso en segundo plano; no se cambió el archivo abierto en la sesión interactiva del usuario. El exportador glTF de esta distribución omite un módulo de animación: se utiliza la misma solución local para exportación estática que ya existe en el script de preparación de la nave Butters. No se modificó la instalación de Blender, ni el script o modelo de la nave, y la exportación de animaciones está desactivada explícitamente.

## Conversión

El generador importa una copia de los objetos del `.blend`, evalúa primero sus jerarquías y recupera la orientación local eliminando la base radial de cada ancla. Todos los hijos se transforman junto con su edificio: ventanas, rótulos, escaparates y detalles de esquina mantienen su relación con las fachadas. Se invierte la proyección radial de las cimentaciones v2/v3 por vértice; también se elimina la curvatura local de la base de las montañas. Se recalculan normales únicamente sobre las copias.

Se conservan **160 edificios** (118 casas, 34 comercios y 8 edificios públicos/hitos), **220 pinos**, **14 montañas** y el cartel con texto de malla. Se usan **27 de los 28 materiales originales** sin cambios: solo se omite el material del lago. No se reconstruyen manualmente edificios. El lago esférico no forma parte de esta adaptación; no es necesario para el recorrido urbano inicial.

El mapa se despliega mediante `X=(mapX−800)×0.36`, `Y=(280−mapY)×0.55`, en metros. Esto conserva barrios, avenidas, orientación de fachadas e hitos y deja espacio adicional entre parcelas. Se usan las 15 rutas de los generadores históricos como polilíneas de calles planas de 6 m de ancho, con marcas amarillas y estacionamiento delante del centro comercial. Las capas visuales tienen diferencias milimétricas de altura para evitar parpadeos en las intersecciones. La nieve es un suelo horizontal de 720×335 m con superficie Z=0. La geometría original de carreteras, que pertenece a una esfera, no se exporta.

Los edificios y pinos se escalan por 1.75; las puertas residenciales miden entre 2.06 y 2.19 m. El ojo inicial está a 1.70 m. El cartel se reduce respecto de su exageración planetaria y las montañas se amplían como fondo. Son recursos de estilo caricaturesco, sin interiores ni precisión arquitectónica. El ajuste determinista de implantación conserva la posición de los edificios y desplaza solo los elementos que invadirían una calle u otro objeto; el detalle de cada desplazamiento queda en `relocations` del manifiesto. En la generación final no fueron necesarios desplazamientos adicionales.

La iluminación de desarrollo combina cielo invernal y sol suave. Los materiales conservan sus nodos y colores, sin texturas que descargar o reubicar.

## Abrir y recorrer

1. Abrir `assets/models/minigames/southpark/southpark_town.blend` en una ventana de Blender.
2. Usar **Numpad 0** para entrar en la cámara de calle si no está activa.
3. Con el cursor sobre el visor, usar **F3 → Walk Navigation** (también `Shift + acento grave` en teclados compatibles).
4. Moverse con WASD y mirar con el ratón. Q/E permiten ajustar la altura en la navegación sin gravedad. Consultar la barra de estado para los controles de la configuración local. Clic confirma el recorrido; Esc lo cancela y restaura la vista.
5. Para revisar todo el pueblo, salir de cámara y usar Home / Frame All con la colección visual seleccionada.

Se utiliza la navegación incorporada de Blender como herramienta de desarrollo. No tiene las colisiones futuras del juego ni un límite que impida atravesar edificios o salir del terreno. Evitar activar gravedad sin configurar antes la navegación local; la altura de referencia es la cámara guardada a 1.70 m.

Capturas generadas con los materiales reales: [calle](../references/southpark/phase1/street.png) y [vista general](../references/southpark/phase1/overview.png).

## Regenerar y validar

Desde la raíz del repositorio, sustituyendo `blender` por la ruta local del ejecutable si no está en PATH:

```powershell
blender --background --python-exit-code 1 --python tools/blender/southpark/build_explorable_town.py
blender --background --python-exit-code 1 --python tools/blender/southpark/validate_explorable_town.py
```

El generador reemplaza solamente el `.blend`, GLB, manifiesto y capturas derivados. No lo ejecutar sobre una copia derivada con ediciones manuales que se quieran conservar. El planeta se abre como biblioteca de solo lectura; nunca se guarda en su ruta original. Su SHA-256 esperado es `960590e3c4a2a564f35b7a49fcd499d94e35017460704815b0fecd7af93557e9`.

El validador vuelve a abrir el `.blend` entregado, comprueba las 395 anclas reutilizadas, materiales, ausencia de dependencias externas, alineación vertical, apoyo en suelo, escala de puertas, límites, separación de edificios y punto inicial. Muestrea tres bandas de marcha a lo largo de todas las calles para un jugador de 70 cm de diámetro y revisa la conectividad del trazado en una cuadrícula. Reimporta el GLB y compara objetos y límites mundiales para detectar errores de posición, escala o conversión de ejes. Guarda [validation.json](../references/southpark/phase1/validation.json); falla con código distinto de cero ante errores. Estas verificaciones de geometría no sustituyen pruebas de un controlador con colisiones.

No hay suite de pruebas del juego ni comandos de build previos. La conservación de recursos existentes se verifica mediante SHA-256 y el diff de Git; las pruebas del sistema solar deberán hacerse cuando exista su implementación.

## Archivos y fase 2

Todos los archivos de esta fase son nuevos: los tres recursos y README en `assets/models/minigames/southpark/`, el contrato en `src/minigames/southpark/README.md`, esta guía, los dos scripts `build_explorable_town.py` / `validate_explorable_town.py` y las capturas e informe bajo `docs/references/southpark/phase1/`. El planeta esférico, las naves y la documentación original se conservan.

La fase 2 necesita acordar el motor y el adaptador del [contrato de escena](../../src/minigames/southpark/README.md), implementar el controlador FPS y las colisiones sencillas, probar altura, velocidad, puertas y límites con el jugador real, y revisar las cajas conservadoras en porches y montañas. La geometría editable prioriza conservación y revisión; antes de integrar conviene compartir las mallas repetidas y evaluar agrupación por material, draw calls y rendimiento en el motor elegido. Los materiales de colores planos admiten iluminación distinta, pero el motor deberá reproducir el ambiente nevado del visor.

Los personajes, NPC, mecánicas SAVE KENNY!, amenazas, interacción, HUD, tiempo, vidas, puntos e integración definitiva con el sistema solar siguen fuera de esta entrega. No se hicieron commits ni push ni se cambió la rama.
