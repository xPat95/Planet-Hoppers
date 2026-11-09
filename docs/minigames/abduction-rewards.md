# Abducciones, tokens y naves

`GameState` conserva por separado la nave inicial, la activa, las naves poseídas y
los planetas completados. `rewards::record_result` sigue siendo el único punto de
entrada para las recompensas de los seis minijuegos. No se modificaron sus reglas.

`TokenPresentation` coordina una cola FIFO:

- Selección inicial: abducción de 4,8 segundos, sin token ni aviso de nave.
- Primera victoria con nave bloqueada: abducción → ceremonia existente de token
  (3,6 segundos) → aviso de nave (4 segundos).
- Primera victoria con nave ya poseída: únicamente ceremonia de token.
- Repeticiones, derrotas y abandonos: no añaden etapas.

El aviso no bloquea el vuelo. Las etapas cinematográficas usan el bloqueo de
controles y la restauración del cursor del medallero. El último frame de una
cinemática también descarta entradas; al recuperar el mouse se descartan sus
primeros deltas. La cola espera a que terminen el fundido y la entrada al sistema
solar antes de comenzar, y no se superpone con ayuda ni medallero.

Las abducciones muestran las naves reales y los personajes originales: Steve;
Stan, Kyle, Kenny y Cartman; Pim y Charlie; Grogu; Ghost Face; Mordecai y Rigby.
Cada grupo usa su PNG de `assets/images/abductions/`, recortado al centro con
escala de cobertura, sin estirar. La animación combina levitación, giro,
convergencia del grupo, reducción hacia el haz, partículas y salida de la nave.
Los colores del haz y la cadencia del grupo distinguen las franquicias.

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
