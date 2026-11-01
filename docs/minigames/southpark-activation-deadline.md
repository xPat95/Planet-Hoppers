# Límite total de activación de amenazas

La partida normal utiliza **un único máximo de 15 segundos**, desde el inicio
del desafío o la salvada anterior hasta que comienza el movimiento del peligro.
No es un plazo adicional por etapa. La exploración libre antes de aceptar el
desafío y las cinemáticas siguen funcionando como antes.

Antes se acumulaban 6 s de intervalo + 0,4 s de recolocación + hasta 70 s de
búsqueda + 2 s de preparación. Ahora `activation_elapsed` cuenta todas esas
fases en `SaveKennySession`. También incluye el aviso de éxito de 1,3 s antes
del siguiente evento. Un reintento abre otro presupuesto tras la secuencia de
muerte, antes de recuperar los controles.

La búsqueda reserva los segundos de preparación dentro del tiempo restante:

```text
búsqueda = min(estimación_por_ruta, 15 - acumulado - preparación)
```

Con la configuración actual, el primer evento tiene aproximadamente
`6 + 0,4 + 6,6 + 2 = 15 s`. Después de una salvada:
`1,3 + 6 + 0,4 + 5,3 + 2 = 15 s`.

Acercarse todavía permite adelantar la preparación. Al vencer el máximo, la
partida pasa a Active y actualiza el director de amenazas en ese mismo frame:
no espera una actualización adicional con el peligro congelado. El piano
empieza su oscilación y la marcha de Kenny; su caída sigue obedeciendo la
mecánica existente. El autobús, abducción, alcantarilla y perseguidor conservan
sus desplazamientos y consecuencias.

El reloj usa el delta real del frame, aunque las físicas limiten su paso a 0,1 s.
Así, los frames lentos no conceden tiempo adicional. La presentación visual
observa la activación en el primer frame que cruza el plazo; el movimiento de
esa actualización ya está aplicado. No se modifican el intro ni sus relojes.

M y P suspenden el reloj junto con el mundo. Cerrarlos conserva el tiempo ya
consumido; no abre otro presupuesto. La configuración admite reducir el máximo,
pero rechaza valores superiores a 15 s y aplica además ese tope en ejecución.

Las velocidades y duraciones activas permanecen intactas: autobús 10 s, OVNI y
alcantarilla 12 s, piano 11 s, muñeco 2,35 m/s y jugador corriendo a 8 m/s.
Tampoco cambian vidas, salvadas, orden aleatorio, raycasting ni diálogos.

## Verificaciones

- `cargo check --locked`, `cargo build --locked` y Clippy sin advertencias.
- `cargo test --locked`: 90 pruebas correctas.
- 100 casos de activación: cinco amenazas × cinco zonas × cuatro pasos de tiempo
  (30, 60 y 144 FPS, además de 0,073 s). Comprueban el reloj desde `start`, no
  solo desde la búsqueda, y exigen peligro avanzando al alcanzar Active.
- Pruebas específicas de frames largos, pausa/mapa sin reinicio, aviso de éxito
  incluido y reintento después de perder una vida.
- 25 fracasos por inactividad y 25 rescates con movimiento, colisiones y raycasting
  desde accesos cercanos a las zonas.
- Partida gráfica automatizada con OpenGL y eventos nativos: victoria, derrota,
  reintentos y cinco comprobaciones de pausa/reanudación del mapa.

Resultados: [reporte del límite de activación](../references/southpark/activation-deadline/validation.json).
Las pruebas gráficas fueron silenciadas y no sustituyen la revisión manual de
controles y dificultad.

La ciudad real conserva recorridos que pueden exceder 30 s a la velocidad
actual, especialmente con rodeos. El plazo no se extiende por esa distancia:
las pruebas de rescate cercanas verifican las mecánicas, no garantizan que un
jugador al otro extremo llegue a todas las amenazas. La dificultad humana sigue
requiriendo una partida manual. No se ralentizan amenazas ni se teletransporta
al jugador para compensarlo.
