# Plan de desarrollo

Las etapas se seguirán en el orden indicado. La siguiente tabla registra el estado inicial del plan: todas las etapas quedan **Pendiente** para su seguimiento y revisión por el equipo. Esta entrega aporta la estructura y documentación de la primera etapa.

| Etapa | Nombre | Descripción | Estado inicial |
| --- | --- | --- | --- |
| 1 | Definición de estructura | Preparar y revisar carpetas, responsabilidades y reglas de separación modular. | Pendiente |
| 2 | Creación de planetas | Crear los seis planetas temáticos y organizar sus modelos y texturas por franquicia. | Pendiente |
| 3 | Creación de naves | Crear las seis naves temáticas y organizar sus recursos y propiedades. | Pendiente |
| 4 | Implementación del sistema solar | Integrar el sol central y los seis planetas en la escena espacial. | Pendiente |
| 5 | Navegación y órbitas | Implementar pilotaje libre, cámara, órbitas, rotación propia e interacción por proximidad. | Pendiente |
| 6 | Personajes | Integrar modelos y personajes de cada franquicia y sus asociaciones con naves; desarrollar la carga compartida, la abducción inicial, las abducciones únicamente por nuevos desbloqueos y la devolución final. La nave y los personajes de Un Show Más siguen pendientes de definir. | Pendiente |
| 7 | Minijuegos | Desarrollar los seis minijuegos y sus componentes comunes, incluido el shooter FPS con raycasting de Un Show Más (`regularshow`), basado en disparar a enemigos y acumular puntos; mantener Star Wars sin raycasting. | Pendiente |
| 8 | Menús, selección de naves y estación espacial | Integrar menú espacial, plataforma iluminada y giratoria y selección inicial libre de una única nave, dejando bloqueadas las otras cinco. Incorporar estación, interacción, hangar, visualización de las seis naves bloqueadas o desbloqueadas y cambio voluntario solo entre las desbloqueadas. Al salir, continuar con la nave seleccionada. | Pendiente |
| 9 | Recompensas y progreso | Registrar misiones completadas y tokens únicos por planeta; coordinar desbloqueos de nuevas naves y abducciones solo por nuevos desbloqueos, evitando recompensas y animaciones duplicadas. La nave inicial ya está desbloqueada, pero su misión aún puede entregar su token. Mantener la nave activa al regresar de los minijuegos, permitir reintentos tras perder y activar el final con seis tokens independientemente de la nave activa. | Pendiente |
| 10 | Agujero negro y final | Transformar el sol en agujero negro, animar la absorción de planetas y nave y devolver los personajes a su origen. | Pendiente |

El equipo actualizará los estados conforme revise y complete cada etapa. Los responsables de los minijuegos se detallan en [responsabilidades](responsibilities.md); los sistemas generales se desarrollarán de forma compartida. La estación espacial queda compartida y pendiente de asignación, integrada en la etapa 8; su progreso se coordina con las etapas 6 y 9, sin agregar una etapa adicional.

La estación espacial y las nuevas mecánicas están planificadas, todavía sin implementación.

La elección de lenguaje, motor gráfico, librerías y formato de configuración sigue pendiente. Los comandos de ejecución y las pruebas se documentarán cuando exista implementación.
