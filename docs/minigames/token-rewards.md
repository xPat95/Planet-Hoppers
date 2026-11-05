# Tokens y resultados de intentos

La aventura aplica `gameplay::rewards::record_result` desde el sistema solar mediante
un `ResultObserver` que se pasa por `minigame_host::run_with_ship_and_results`.
El observador recibe el resultado durante la ejecución del minijuego, antes de
que sus controles puedan reiniciar el intento o cerrar la visita. No contiene
recursos gráficos. Los adaptadores independientes conservan sus firmas anteriores
y usan un observador vacío; el acceso solar de desarrollo sin `GameState` no
crea una aventura ni inventa propiedad.

`AttemptOutcome` consulta los estados terminales de las reglas existentes.
`AttemptReporter`, propiedad de cada visita y separado de la sesión reiniciable,
notifica una vez cada estado terminal y se rearma al observar el siguiente
intento no terminal. Una entrega fallida puede reintentarse. El servicio de
recompensas deduplica además por franquicia durante toda la aventura.

- Minecraft: `RushState::Victory`, tras cumplir diamantes y salida con tiempo.
  Su historial de intentos y `exit_since` conservan el comportamiento anterior.
- South Park: `SessionState::Victory`, tras las cinco salvadas. Reconstruir
  `SaveKennySession` no reconstruye `GameState` ni borra la recompensa.
- Smiling Friends: `Step::Victory`, tras completar los encuentros.
- Star Wars: destruir el reactor lleva a `Explosion`, seguido de `Outro` y `Won`.
  La victoria se registra al terminar el desafío, antes del video final; reiniciar,
  abandonar o cerrar durante ese video conserva el token. El retorno sigue
  esperando `Won`. Las escenas sintéticas de `--capture` no notifican recompensas.

`MinigameExit` sigue describiendo navegación. Un retorno con derrota o abandono
no borra una victoria anterior. Halloween y Regular Show no tienen adaptadores
funcionales y no emiten resultados.

El hangar permite recorrer todo el catálogo, pero `Docked` solo confirma naves
que `GameState` posee. La confirmación activa la nave sin desbloquearla. Al volver
al sistema solar, `configure_active` y la búsqueda de `ship_index` sincronizan el
modelo con esa identidad. La recompensa no modifica ninguna selección activa.

Las pruebas de integración ejercitan las reglas reales y el observador sin GPU.
Las comprobaciones gráficas `--smoke-returns`, `--smoke-minecraft`,
`--smoke-starwars` y `--smoke-docking` verifican los recorridos entre ventanas.
El smoke de hangar prepara dos naves poseídas al crear su fixture, sin simular
victorias ni conceder tokens. Esto solo sucede en ese modo de prueba; las pruebas
de recompensas comprueban por separado que las victorias desbloquean la nave.

No se añaden persistencia, interfaz de tokens ni escena final.
