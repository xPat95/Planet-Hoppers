# Despegue seguro de la estación

Corrección en `feature/hangar`, sin cambios en modelos, hangar visual, plataforma, selección, órbitas ni controles de vuelo. No se hizo commit ni push.

## Causa y solución

El antiguo despegue usaba la puerta más 36 unidades: aproximadamente `(0,6032,743)`. Ese punto estaba **dentro** del colisionador esférico de radio 860 centrado en `(0,6200,0)`. La excepción del corredor permitía estar allí, pero salir lateralmente podía bloquear o corregir la nave. Además, una cámara de presentación situada delante de la nave miraba hacia la estación durante 2,5 segundos.

Ahora `Station::departure(DepartureBounds)` recibe los límites ajustados por el mismo cargador de los modelos renderizados. Proyecta la caja orientada de la estación sobre la dirección de salida y toma el máximo entre ese soporte, el radio de colisión y el marco exterior de la compuerta. La caja ajustada real de la estación tiene semiejes `(700,317.357,332.4785)` antes de su rotación global; el colisionador actual es el límite dominante en la dirección de salida.

El volumen de la nave se cubre con la esfera que contiene sus semiejes ajustados, o su radio físico si es mayor. Para el modelo provisional sin GLB se usa una caja conservadora de su longitud configurada. Se reserva también el desplazamiento trasero de la cámara y una esfera que contiene su plano cercano, calculada con FOV, aspecto y near reales. El margen libre mínimo es `max(departure_clearance, 2*radio_nave, 2*radio_cámara)`; actualmente son 36 unidades.

La posición se obtiene avanzando desde la puerta transformada hasta superar todo ese límite. Una búsqueda acotada de hasta 16 pasos adicionales comprueba por separado nave, cámara y segmento de avance. Exige estar fuera de la esfera y del plano exterior del modelo, además de pasar `push_out`: la exención del corredor por sí sola nunca valida el despegue. Si no encuentra una posición válida devuelve un error, sin desactivar colisiones.

La orientación usa `Ship::look_at` y el vector forward del controlador, conservando las correcciones visuales de cada GLB. Con la configuración actual apunta a `+Z` y la estación queda detrás. Las salidas casi verticales respetan el límite de pitch existente (±1,4 radianes) y se validan con la dirección efectiva del controlador. No se cambió la física de vuelo.

La nave se crea con velocidad, arrastre y ladeo reiniciados **antes** de inicializar la cámara y la posición física anterior. La cámara comienza directamente en la persecución normal, detrás de la nave. Se conserva la protección de reentrada de 1,2 segundos y la obligación de una pulsación nueva. `dock_target()` conserva el punto cercano exclusivamente para aproximarse y entrar al hangar.

## Resultados reales con la configuración actual

Estos valores se calculan; no son constantes de posición añadidas al código. X es aproximadamente cero; Y de la nave es 6032. La cámara comienza en Z=897, con la altura habitual de cada nave.

| Nave / identidad | Radio visual conservador | Z de despegue | Distancia desde la puerta |
| --- | ---: | ---: | ---: |
| Calaca / halloween | 3,464 | 909 | 202 |
| N1 / starwars | 2,865 | 912 | 205 |
| Smiling Friends | 4,408 | 915 | 208 |
| Minecraft | 3,372 | 912 | 205 |
| Golf / regularshow | 2,723 | 909 | 202 |
| Butters / southpark | 2,915 | 909 | 202 |

Smiling Friends es la nave con mayor envolvente ajustada de vuelo. La cámara y toda la nave quedan por delante de la envolvente exterior, con margen, sin alejarse innecesariamente de la estación.

## Verificación

Pasaron `cargo check --locked`, `cargo build --locked` y las **29 pruebas** de:

```powershell
cargo test --locked --test station_departure --test station --test ship_selection --test ships --test minigame_returns
```

Los cuatro tests nuevos comprueban límites de los seis modelos reales, orientación del controlador y nariz renderizada, plano cercano de cámara, posición estable sin acelerar, avance sin colisión y repetibilidad. Incluyen 216 combinaciones de rotación, traslación, escala y tamaño de colisionador, también con un colisionador menor que el modelo visual. Los tests existentes conservan entrada, corredor, selección y regresos de minijuegos.

Se ejecutó el juego nativo con automatización de raylib para las seis naves, pasando selección y despegue. Las verificaciones en los primeros frames comprueban cámara detrás y sin corrección inicial, estación detrás, ausencia de golpes, ausencia de desplazamiento sin acelerar y avance con W. También pasó `--smoke-docking --ship minecraft`: tres regresos con la misma nave, cambio a N1 y cancelación, seguidos de nuevos despegues sin acumulación de posición. Los logs contienen `DEPARTURE_SAFE`, `LAUNCH_SMOKE_PASS` y `DOCKING_SMOKE_PASS`; [resumen de ejecución](runtime-summary.json).

Capturas de vuelo real, con la estación a espaldas de la cámara y espacio libre delante:

- [N1](launch-starwars.png), [Smiling Friends (mayor tamaño)](launch-smilingfriends.png), [Minecraft](launch-minecraft.png).
- [Calaca](launch-halloween.png), [Golf](launch-regularshow.png), [Butters](launch-southpark.png).
- [Primer despegue del ciclo](dock-launch-0.png), [regreso sin cambio](dock-launch-1.png), [cambio de nave](dock-launch-2.png), [cancelación](dock-launch-3.png).
- [Aproximación de regreso](dock-approach-0.png), [segundo regreso](dock-approach-1.png), [tercer regreso](dock-approach-2.png).

Estas pruebas gráficas usan entrada automatizada; no sustituyen una sesión manual con ratón. El recorrido automatizado de regreso comienza con una colocación de prueba cerca de la aproximación y luego usa navegación e interacción reales; no recorre todo el sistema solar. Las variantes de transformación se probaron geométricamente, sin editar la configuración del juego. Los logs completos de compilación, tests y ejecución están en esta carpeta.

## Comprobación manual

1. Ejecutar `cargo run --locked`, abrir el hangar y seleccionar Smiling Friends.
2. Despegar: comprobar cámara detrás y avanzar con W; no debe haber golpe ni salto inicial. Girar para reconocer la estación detrás. Espacio conserva el ascenso.
3. Seleccionar la estación con Tab, aproximarse con B y W y entrar con una pulsación nueva de E al aparecer la indicación.
4. Cambiar de nave, despegar y repetir; probar también cancelar la selección.

Archivos de implementación: `src/gameplay/station.rs` y `src/scenes/solar_system.rs`. Pruebas: `tests/station.rs` y `tests/station_departure.rs`. Se mantienen intactos modelos, configuración, render del hangar, plataforma, módulo de control de naves y colisiones de estación; estas últimas siguen usando la envolvente esférica y el corredor existentes, no colisiones por triángulo.
