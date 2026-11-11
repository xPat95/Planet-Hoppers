# Abducciones, tokens y naves

`GameState` conserva por separado la nave inicial, la activa, las naves poseídas y
los planetas completados. `rewards::record_result` sigue siendo el único punto de
entrada para las recompensas de los seis minijuegos. No se modificaron sus reglas.

`TokenPresentation` coordina una cola FIFO:

- Selección inicial: abducción de 4,8 segundos, sin token ni aviso de nave. Se ve en
  el hangar apenas se confirma la nave, antes de cargar el sistema solar (la partida
  llega al sistema solar con esa abducción ya marcada como vista). Solo el binario de
  desarrollo `solarsystem`, que no pasa por el hangar, la muestra en el sistema solar.
- Primera victoria con nave bloqueada: abducción (salvo Smiling Friends, cuyo video
  solo se ve al elegir la nave en el hangar: `returns_abducted`; la de Hexbound es otra
  escena, de 3 segundos: la carta de agradecimiento, `assets/ui/halloween/thanks.png`,
  sola en lo negro con la luz cálida, que aparece con un saltito y flota entre corazones
  que suben flotando y laten, unos atrás y otros adelante: `draw_thanks`) →
  ceremonia existente de token
  (3,6 segundos) → aviso de nave (4 segundos).
- Primera victoria con nave ya poseída: únicamente ceremonia de token.
- Repeticiones, derrotas y abandonos: no añaden etapas.

El aviso no bloquea el vuelo. Las etapas cinematográficas usan el bloqueo de
controles y la restauración del cursor del medallero. El último frame de una
cinemática también descarta entradas; al recuperar el mouse se descartan sus
primeros deltas. La cola espera a que terminen el fundido y la entrada al sistema
solar antes de comenzar, y no se superpone con ayuda ni medallero.

raylib junta lo que se dibuja en 2D y lo manda después, y estas escenas borran la
pantalla en medio del cuadro del sistema solar: por eso, antes de dibujarlas, se manda
lo que ya estaba dibujado (`flush_2d` en `abduction.rs`); si no, las etiquetas y los
logos de los planetas quedaban encima de la escena.

Las abducciones muestran las naves reales y los personajes originales: Steve;
Stan, Kyle, Kenny y Cartman; Pim y Charlie; Grogu; Ghost Face; Mordecai y Rigby.
Cada grupo usa su PNG de `assets/images/abductions/`, recortado al centro con
escala de cobertura, sin estirar. La animación combina levitación, giro,
convergencia del grupo, reducción hacia el haz, partículas y salida de la nave.
Los colores del haz y la cadencia del grupo distinguen las franquicias.

La abducción de **Smiling Friends** es distinta: un video a pantalla completa, sin
marco ni franjas (recortado al centro, sin estirar), con su propio sonido
(`assets/video/smilingfriends/abduction.sfv` y `abduction.ogg`, de
`source/abduction.mov`). Dura lo que el video (311 cuadros a 24 por segundo, unos
13 segundos; `abduction_seconds` da la duración de cada franquicia) y se puede
saltear con el mismo botón **SKIP** de la intro del minijuego, o con Espacio, Enter
o Esc. Mientras suena, la música del menú se calla y después sigue desde donde
quedó. Solo se ve al elegir esa nave en el hangar (al ganar el minijuego no hay abducción).

La de **Hexbound** tampoco es una abducción sino la invitación a la fiesta (3,1
segundos), con la música de la intro del minijuego ("Exploring", que se apaga con el fundido
final; la del menú se calla mientras tanto), dibujada con el arte del minijuego (los mismos modelos, sombras, negro y
lente con grano, `draw_invitation` en `src/minigames/halloween/story.rs`) pero con
una luz más suave que la del juego, elegida por Cami entre varias pruebas: **cálida y
suave** (`INVITATION_LIGHT`): la luz cálida del farol de arriba y del lado de la
cámara, un relleno suave para que nada quede negro y apenas el borde frío. Primero la
carta de cerca (el dibujo de Cami, `assets/ui/halloween/invitation.png`, 1200×1500
px) en lo negro; después, una transición de círculo (iris, como en los dibujos
animados viejos: `iris.fs`, sin borde) que se abre en el medio
y la deja ver a ella en su escoba: la nave tal cual (la bruja con la mano levantada,
la escoba y la estela, que acá brilla más), flotando y nunca quieta (sube y baja y se mece, cabecea y gira con varios ritmos
encimados, no uno solo parejo; el
brazo levantado saluda desde el hombro y el codo, y la parte de atrás de la falda
flamea con el viento: se le mueven los vértices en el código, porque la nave es una
sola pose), con el papelito (`note.glb`: una hoja en blanco doblada al medio) en la
mano levantada. Es un solo plano que gira, desde abajo (mirándola hacia arriba, como
en el cine): la cámara empieza de tres cuartos de frente, del lado de la mano
levantada, y la rodea en menos de un segundo hasta quedar detrás, todavía un poco
abajo. Ahí el papelito se le vuela (el viento lo lleva para atrás, dando vueltas), ella deja
de saludar y baja el brazo al costado (desde el hombro, estirando el codo), y
ella sale disparada hacia el fondo, casi derecha, arrancando suave y acelerando,
culebreando de un lado al otro (ladeándose hacia cada curva) y con sacudones de viento, con
la estela hacia la cámara, hasta perderse en lo oscuro. Para esto, la luz del
minijuego (`Light` en `render.rs`) tiene además un relleno parejo y la intensidad del
borde frío (en el juego, 0 y 1: no cambia), y la invitación puede llevar un cielo
(`sky`, sin usar ahora).

Los OBJ de personajes referencian MTL que no están en el repositorio. Por eso se
exportaron los `.blend` originales a `assets/models/characters/abductions/*.glb`,
con sus texturas y colores empaquetados. Los originales permanecen intactos.
Para regenerarlos:

```powershell
blender -b --python tools/blender/export_abduction_characters.py -- .
```

El guardado versionado está en `saves/adventure.json` (ignorado por Git). Guarda
el progreso y la cola en una sola escritura temporal, sincronizada y reemplazada
por rename. Se actualiza al iniciar una partida, ganar una recompensa, cambiar
de nave y pasar de etapa. El menú ofrece **[R] CONTINUAR** si existe un guardado.
**JUGAR** inicia una nueva partida y reemplaza ese guardado al confirmar nave.
Continuar vuelve a la estación con la nave activa y el progreso conservados;
no guarda posición de vuelo ni intentos de minijuegos.

Una secuencia interrumpida continúa desde la última etapa guardada: no vuelve a
ejecutar etapas ya terminadas, ni concede otra vez sus recompensas. Si se cierra
a mitad de etapa, esa etapa se reanuda desde su último snapshot. El marcador de
abducción inicial impide programarla al volver al hangar o continuar. Los modos
`--smoke*` están aislados del guardado real.

Verificación:

```powershell
cargo test --locked
cargo test --locked --test abduction_rewards abduction_visual_smoke -- --ignored --nocapture
cargo test --locked --test token_presentation token_visual_smoke -- --ignored --nocapture
```

Las pruebas de lógica cubren las 36 combinaciones de nave inicial y planeta,
repeticiones, orden de etapas, estados independientes, bloqueo en hangar,
reemplazo del guardado y recuperación entre sesiones. El smoke gráfico genera
capturas de seis franquicias, cuatro etapas y tres resoluciones en
`target/abductions/`.
